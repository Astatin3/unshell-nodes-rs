use slint::{ComponentHandle, ModelRc, VecModel};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};
use unshell_rs_lib::config::campaign::CampaignConfig;

use crate::{client::UnshellClient, packets::Parameter};

pub struct UnshellGui {
    client: UnshellClient,
    ui: AppWindow,
    campaign: Option<Arc<Mutex<CampaignConfig>>>,
}

slint::include_modules!();
impl UnshellGui {
    pub fn start(client: UnshellClient) -> Result<(), Box<dyn Error>> {
        let ui = AppWindow::new()?;
        let client = Arc::new(Mutex::new(client));

        let ui_handle = ui.as_weak();
        let client_clone = Arc::clone(&client);
        ui.on_tab_clicked(move |index| {
            let ui = ui_handle.unwrap();
            ui.set_current_tab(index);
            info!("Lock 1 ");
            let mut client_lock = client_clone.lock().unwrap();
            info!("Lock 1 ");
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

        ui.run()?;

        Ok(())
    }

    fn update(&mut self) {
        // self.ui.set_listeners(ModelRc::new(VecModel::from(
        //     self.campaign
        //         .listeners
        //         .iter()
        //         .map(|l| match l {
        //             ListenerConfig::Tcp {
        //                 enabled,
        //                 name,
        //                 addr,
        //                 layers,
        //                 ..
        //             } => UITcpListener {
        //                 enabled: *enabled,
        //                 name: name.clone().into(),
        //                 remote_host: addr.to_string().into(),
        //             },
        //         })
        //         .collect::<Vec<UITcpListener>>(),
        // )));
    }
}

// trait
