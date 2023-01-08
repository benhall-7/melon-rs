pub mod config;
pub mod events;
pub mod melon;

fn main() {
    let mut lock = melon::nds::INSTANCE.lock().unwrap();
    let mut ds = lock.take().unwrap();

    ds.load_cart(&std::fs::read("/Users/benjamin/Desktop/ds/Ultra.nds").unwrap(), None);

    println!("{}", ds.cart_inserted());
}
