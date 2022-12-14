use super::sys;

pub fn cart_inserted() -> bool {
    sys::NDS::CartInserted()
}
