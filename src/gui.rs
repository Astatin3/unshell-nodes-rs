use slint::{ModelRc, VecModel};
use std::error::Error;
use unshell_rs_lib::config::listeners::ListenerConfig;

pub struct Unshell_Gui;

slint::include_modules!();
impl Unshell_Gui {
    pub fn start() -> Result<Self, Box<dyn Error>> {
        let ui = AppWindow::new()?;

        // ui.

        let ui_handle = ui.as_weak();
        ui.on_tab_clicked(move |index| {
            let ui = ui_handle.unwrap();
            ui.set_current_tab(index);
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

        let listeners: Vec<ListenerConfig> = vec![ListenerConfig::Tcp {
            enabled: true,
            name: "test".to_string(),
            remote_host: "127.0.0.1".to_string(),
            port: 25565,
            layers: Vec::new(),
        }];

        ui.set_listeners(ModelRc::new(VecModel::from(
            listeners
                .iter()
                .map(|l| match l {
                    ListenerConfig::Tcp {
                        enabled,
                        name,
                        remote_host,
                        port,
                        layers,
                    } => UITcpListener {
                        enabled: *enabled,
                        name: name.clone().into(),
                        remote_host: remote_host.clone().into(),
                        port: *port as i32,
                    },
                })
                .collect::<Vec<UITcpListener>>(),
        )));

        ui.run()?;

        Ok(Self {})
    }
}
