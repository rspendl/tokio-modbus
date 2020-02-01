use futures::Future;

use std::{cell::RefCell, io::Error, rc::Rc};

use std::pin::Pin;
use tokio_modbus::client::{rtu, Context};
use tokio_modbus::prelude::{
    client::util::{reconnect_shared_context, NewContext, SharedContext},
    *,
};

const SLAVE_1: Slave = Slave(0x01);
const SLAVE_2: Slave = Slave(0x02);

#[cfg(feature = "rtu")]
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use tokio_serial::{Serial, SerialPortSettings};

    #[derive(Debug)]
    struct SerialConfig {
        path: String,
        settings: SerialPortSettings,
    }

    impl NewContext for SerialConfig {
        fn new_context(&self) -> Pin<Box<dyn Future<Output = Result<Context, Error>>>> {
            let serial = Serial::from_path(&self.path, &self.settings);
            Box::pin(async {
                let port = serial?;
                rtu::connect(port).await
            })
        }
    }

    let serial_config = SerialConfig {
        path: "/dev/ttyUSB0".into(),
        settings: SerialPortSettings {
            baud_rate: 19200,
            ..Default::default()
        },
    };
    println!("Configuration: {:?}", serial_config);

    // A shared, reconnectable context is not actually needed in this
    // simple example. Nevertheless we use it here to demonstrate how
    // it works.
    let shared_context = Rc::new(RefCell::new(SharedContext::new(
        None, // no initial context, i.e. not connected
        Box::new(serial_config),
    )));

    // Reconnect for connecting an initial context
    reconnect_shared_context(&shared_context).await?;

    assert!(shared_context.borrow().is_connected());
    println!("Connected");

    println!("Reading a sensor value from {:?}", SLAVE_1);
    let context = shared_context.borrow().share_context().unwrap();
    let mut context = context.borrow_mut();
    context.set_slave(SLAVE_1);
    let response = context.read_holding_registers(0x082B, 2).await?;
    println!("Sensor value for device {:?} is: {:?}", SLAVE_1, response);

    println!("Reading a sensor value from {:?}", SLAVE_2);
    context.set_slave(SLAVE_2);
    let response = context.read_holding_registers(0x082B, 2).await?;

    println!("Sensor value for device {:?} is: {:?}", SLAVE_2, response);

    Ok(())
}

#[cfg(not(feature = "rtu"))]
pub fn main() {
    println!("feature `rtu` is required to run this example");
    std::process::exit(1);
}
