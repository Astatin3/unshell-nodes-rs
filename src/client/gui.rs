use slint::{ComponentHandle, Weak};
use std::{
    error::Error,
    sync::{Arc, Mutex},
    thread,
};
use unshell_rs_lib::connection::{C2Packet, Parameter};

use crate::client::UnshellClient;

pub struct UnshellGui {}

slint::include_modules!();
impl UnshellGui {
    pub fn start(client: UnshellClient) -> Result<(), Box<dyn Error>> {
        let ui = AppWindow::new()?;
        let rx = client.rx.clone();
        let client = Arc::new(Mutex::new(client));

        let ui_handle = ui.as_weak();
        let client_clone = Arc::clone(&client);
        ui.on_tab_clicked(move |index| {
            let ui = ui_handle.unwrap();
            ui.set_current_tab(index);
            let mut client_lock = client_clone.lock().unwrap();
            client_lock.set_parameter("Current Tab".to_string(), Parameter::CurrentTab(index));
            trace!("Tab {} selected", index);
        });

        ui.set_app_info({
            (String::new()
                + "Unshell\n"
                + "Version "
                + env!("CARGO_PKG_VERSION")
                + "\n\n View the source code at:\n https://github.com/astatin3/unshell-rs")
                .into()
        });

        let ui_handle = ui.as_weak();
        thread::spawn(move || {
            fn on_param_update(ui_handle: Weak<AppWindow>, parameter: &Parameter) {
                // info!("{}", name);
                match parameter {
                    Parameter::Test1 => todo!(),
                    Parameter::CurrentTab(i) => {
                        let i = i.clone();
                        slint::invoke_from_event_loop(move || {
                            ui_handle.unwrap().set_current_tab(i)
                        })
                        .unwrap();
                    }
                }
            }

            loop {
                if let Ok(data) = rx.recv() {
                    match data {
                        C2Packet::SetAllParameters(parameters) => {
                            for key in parameters.keys() {
                                on_param_update(ui_handle.clone(), parameters.get(key).unwrap());
                            }
                        }
                        C2Packet::ParameterUpate(name, parameter) => {
                            on_param_update(ui_handle.clone(), &parameter);
                        }
                        _ => {}
                    }
                }
            }
        });

        ui.run()?;

        Ok(())
    }
}
