macro_rules! make_message {
    ( $( $bytes:literal ),* ) => {
        // All launchpad SysEx messages start and end the same way
        &[0xf0, 0x00, 0x20, 0x29, 0x02, 0x0e, $($bytes),*, 0xf7]
    };
}

pub(super) const ENTER_LIVE: &[u8] = make_message!(0x0e, 0x00);
pub(super) const ENTER_PROGRAMMER: &[u8] = make_message!(0x0e, 0x01);
