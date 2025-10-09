use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
	pub epoch_ms: u128,
	pub active_layer: u8,
	pub pressed_bits: u64, // lower 48 bits used for 4x12
}

impl Report {
	pub const PLANCK_NUM_KEYS: usize = 48;

	pub fn now(active_layer: u8, pressed_bits: u64) -> Self {
		let epoch_ms = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.map(|d| d.as_millis())
			.unwrap_or(0);
		Self { epoch_ms, active_layer, pressed_bits }
	}
}

pub trait HidSource {
	fn poll(&mut self) -> Option<Report>;
}

pub fn parse_rawhid_packet(bytes: &[u8]) -> Option<Report> {
	// Simple protocol: [layer: u8][pressed_bits: u64 little-endian] -> we only use 6 LSB bytes
	if bytes.len() < 7 { return None; }
	let active_layer = bytes[0];
	let mut buf = [0u8; 8];
	buf[..6].copy_from_slice(&bytes[1..7]);
	let pressed_bits = u64::from_le_bytes(buf);
	Some(Report::now(active_layer, pressed_bits))
}

pub struct MockHidSource {
	counter: u64,
}

impl MockHidSource {
	pub fn new() -> Self { Self { counter: 0 } }
}

impl HidSource for MockHidSource {
	fn poll(&mut self) -> Option<Report> {
		self.counter = self.counter.wrapping_add(1);
		let layer = ((self.counter / 120) % 4) as u8; // cycle layers every ~1s
		let idx = (self.counter % Report::PLANCK_NUM_KEYS as u64) as usize;
		let mut bits = 0u64;
		bits |= 1u64 << idx; // single moving key
		Some(Report::now(layer, bits))
	}
}

#[cfg(feature = "rawhid")]
pub struct RawHidSource {
	ctx: hidapi::HidApi,
	// We lazily open device by vendor/product or usage page; for now keep optional handle
	device: Option<hidapi::HidDevice>,
}

#[cfg(feature = "rawhid")]
impl RawHidSource {
	pub fn new() -> Self {
		let ctx = hidapi::HidApi::new().expect("hidapi init");
		Self { ctx, device: None }
	}

	fn ensure_device(&mut self) {
		if self.device.is_some() { return; }
		let mut fallback: Option<hidapi::HidDevice> = None;
		for dev in self.ctx.device_list() {
			let product = dev.product_string().unwrap_or_default();
			let prod_lc = product.to_lowercase();
			if prod_lc.contains("planck") || prod_lc.contains("qmk") {
				if let Ok(d) = dev.open_device(&self.ctx) { self.device = Some(d); return; }
			}
			if fallback.is_none() {
				if let Ok(d) = dev.open_device(&self.ctx) { fallback = Some(d); }
			}
		}
		if self.device.is_none() { self.device = fallback; }
	}
}

#[cfg(feature = "rawhid")]
impl HidSource for RawHidSource {
	fn poll(&mut self) -> Option<Report> {
		self.ensure_device();
		let Some(dev) = self.device.as_ref() else { return None; };
		let mut buf = [0u8; 64];
		match dev.read_timeout(&mut buf, 1) {
			Ok(n) if n > 0 => parse_rawhid_packet(&buf[..n]),
			_ => None,
		}
	}
}

#[cfg(feature = "qmk_console")]
pub struct QmkConsoleSource {
	port: Option<Box<dyn serialport::SerialPort>>,
	buf: String,
	last_try: std::time::Instant,
	override_port: Option<String>,
}

#[cfg(feature = "qmk_console")]
impl QmkConsoleSource {
	pub fn new() -> Self { Self::new_with_port(None) }
	pub fn new_with_port(port: Option<String>) -> Self {
		Self { port: None, buf: String::new(), last_try: std::time::Instant::now(), override_port: port }
	}

	fn open_port_name(&self, name: &str) -> Option<Box<dyn serialport::SerialPort>> {
		serialport::new(name, 115_200)
			.timeout(std::time::Duration::from_millis(1))
			.open()
			.ok()
	}

	fn ensure_port(&mut self) {
		if self.port.is_some() { return; }
		if self.last_try.elapsed().as_millis() < 500 { return; }
		self.last_try = std::time::Instant::now();
		if let Some(name) = self.override_port.clone() {
			self.port = self.open_port_name(&name);
			return;
		}
		if let Ok(ports) = serialport::available_ports() {
			for p in ports {
				let name = p.port_name.to_lowercase();
				if name.contains("usbmodem") || name.contains("usbserial") {
					if let Some(port) = self.open_port_name(&p.port_name) {
						self.port = Some(port);
						break;
					}
				}
			}
		}
	}

	fn try_read_line(&mut self) -> Option<String> {
		let Some(port) = self.port.as_mut() else { return None; };
		let mut buf_bytes = [0u8; 128];
		match port.read(&mut buf_bytes) {
			Ok(n) if n > 0 => {
				self.buf.push_str(&String::from_utf8_lossy(&buf_bytes[..n]));
				if let Some(pos) = self.buf.find('\n') {
					let line = self.buf.drain(..=pos).collect::<String>();
					return Some(line.trim().to_string());
				}
			}
			_ => {}
		}
		None
	}
}

#[cfg(feature = "qmk_console")]
impl HidSource for QmkConsoleSource {
	fn poll(&mut self) -> Option<Report> {
		self.ensure_port();
		if let Some(line) = self.try_read_line() {
			// Log raw line for debugging
			eprintln!("console: {}", line);
			let mut layer: Option<u8> = None;
			let mut bits: Option<u64> = None;
			for part in line.split_whitespace() {
				if let Some(val) = part.strip_prefix("L:") {
					layer = val.parse::<u8>().ok();
				} else if let Some(val) = part.strip_prefix("B:") {
					bits = u64::from_str_radix(val, 16).ok();
				}
			}
			if let (Some(l), Some(b)) = (layer, bits) {
				let rep = Report::now(l, b);
				eprintln!("parsed: layer={} bits=0x{:012X}", rep.active_layer, rep.pressed_bits);
				return Some(rep);
			}
		}
		None
	}
}
