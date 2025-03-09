use std::{fs::File, io::Read, path::Path, time::Duration};

use cnx::{
    text::{Attributes, Text},
    widgets::Widget,
};
use tokio::time;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;

pub struct Battery {
    attrs: Attributes,
    render: Option<Box<dyn Fn(BatteryInfo) -> String>>,
    update_interval: Duration,
    battery_path: String,
}

/// Battery statuses as written in `power_supply.h`
#[derive(Debug)]
pub enum ChargeStatus {
    Unknown,
    Charging,
    Discharging,
    NotCharging,
    Full,
}

pub struct BatteryInfo {
    pub status: ChargeStatus,
    pub capacity: u64,
    pub time_till_empty: Duration,
}

impl Battery {
    pub fn new(
        attrs: Attributes,
        render: Option<Box<dyn Fn(BatteryInfo) -> String>>,
        update_interval: Duration,
        battery_path: String,
    ) -> Self {
        Self {
            attrs,
            render,
            update_interval,
            battery_path,
        }
    }

    pub fn tick(&self) -> Vec<Text> {
        let battery_location = Path::new(&self.battery_path);
        let mut current_now =
            File::open(battery_location.join("current_now")).expect("Could not open current_now");
        let mut charge_now =
            File::open(battery_location.join("charge_now")).expect("Could not open charge_full");
        let mut capacity =
            File::open(battery_location.join("capacity")).expect("Could not open capacity");
        let mut status =
            File::open(battery_location.join("status")).expect("Could not open status");

        let mut temp_str = String::new();

        let _ = current_now.read_to_string(&mut temp_str);
        let current_micro_amps: u64 = temp_str
            .trim()
            .parse()
            .expect("File did not contain integer data");

        temp_str.clear();
        let _ = charge_now.read_to_string(&mut temp_str);
        let charge_micro_amp_hrs: u64 = temp_str
            .trim()
            .parse()
            .expect("File did not contain integer data");

        temp_str.clear();
        let _ = capacity.read_to_string(&mut temp_str);
        let current_percent: u64 = temp_str
            .trim()
            .parse()
            .expect("File did not contain integer data");

        temp_str.clear();
        let _ = status.read_to_string(&mut temp_str);
        let batt_status = match temp_str.trim() {
            "Charging" => ChargeStatus::Charging,
            "Discharging" => ChargeStatus::Discharging,
            "Full" => ChargeStatus::Full,
            "Not Charging" => ChargeStatus::NotCharging,
            _ => ChargeStatus::Unknown,
        };

        let estimated_hrs_left = charge_micro_amp_hrs as f64 / current_micro_amps as f64;
        let estimated_duration = Duration::from_secs_f64(3600.0 * estimated_hrs_left);

        let batt_info = BatteryInfo {
            status: batt_status,
            capacity: current_percent,
            time_till_empty: estimated_duration,
        };

        let text = if let Some(render) = &self.render {
            render(batt_info)
        } else {
            format!(
                "{:?} : {}%, : {:.0?}",
                batt_info.status, batt_info.capacity, batt_info.time_till_empty
            )
        };

        vec![Text {
            attr: self.attrs.clone(),
            text,
            stretch: false,
            markup: self.render.is_some(),
        }]
    }
}

impl Widget for Battery {
    fn into_stream(self: Box<Self>) -> anyhow::Result<cnx::widgets::WidgetStream> {
        let interval = time::interval(self.update_interval);
        let stream = IntervalStream::new(interval).map(move |_| Ok(self.tick()));

        Ok(Box::pin(stream))
    }
}
