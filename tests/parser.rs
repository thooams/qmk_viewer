use qmk_viewer::hid::{parse_rawhid_packet, Report};
use qmk_viewer::planck::{PlanckKeyboard, PlanckLayoutState};

#[test]
fn parse_packet_ok() {
	let layer = 2u8;
	let bits: u64 = 0x0000_0000_0000_A55A;
	let mut pkt = vec![layer];
	let le = bits.to_le_bytes();
	pkt.extend_from_slice(&le[..6]);
	let rep = parse_rawhid_packet(&pkt).expect("parsed");
	assert_eq!(rep.active_layer, layer);
	assert_eq!(rep.pressed_bits & 0xFFFF_FFFF_FFFF, bits & 0xFFFF_FFFF_FFFF);
}

#[test]
fn mapping_and_pressed() {
	let kb = PlanckKeyboard::default();
	let mut st = PlanckLayoutState::new(kb);
	// Press row 1, col 3
	let idx = st.index_for(1, 3).unwrap();
	let bits = 1u64 << idx;
	st.set_pressed_bits(bits);
	assert!(st.is_pressed(1, 3));
	assert!(!st.is_pressed(0, 0));
}

#[test]
fn layer_set() {
	let kb = PlanckKeyboard::default();
	let mut st = PlanckLayoutState::new(kb);
	st.set_layer(3);
	assert_eq!(st.active_layer, 3);
}

