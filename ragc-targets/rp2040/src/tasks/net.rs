use embassy::executor::Spawner;
use embassy::util::Forever;
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack, StackResources};
use heapless::Vec;

use crate::Device;

static STACK_RES: Forever<StackResources<1, 2, 8>> = Forever::new();
static STACK: Forever<Stack<Device>> = Forever::new();

#[embassy::task]
async fn net_task(stack: &'static Stack<Device>) -> ! {
    stack.run().await
}

#[embassy::task]
async fn test() {
    loop {}
}

pub fn init_net(spawner: &Spawner, dev: Device, seed: u64) -> &'static Stack<Device> {
    let device = dev;

    let config = embassy_net::ConfigStrategy::Static(embassy_net::Config {
        address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
        dns_servers: Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    });

    // Init network stack
    let stack_res = STACK_RES.put(StackResources::<1, 2, 8>::new());
    let stack_data = Stack::new(device, config, stack_res, seed);

    let stack = STACK.put(stack_data);
    let _ = spawner.spawn(net_task(stack));
    stack
}
