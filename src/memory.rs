use anyhow::Result;
use byte_unit::{Byte, UnitType};
use cnx::text::{Attributes, Text};
use cnx::widgets::{Widget, WidgetStream};
use std::time::Duration;
use sysinfo::{MemoryRefreshKind, System};
use tokio::time;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;

// Abstracted type to represent the render closure
type MemoryRender = Box<dyn Fn((Byte, Byte), (Byte, Byte)) -> String>;

/// cnx widget that shows current system memory usage
pub struct MemoryUsage {
    attrs: Attributes,
    render: Option<MemoryRender>,
    memory_handle: System,
    update_interval: Duration,
}

impl MemoryUsage {
    /// Creates a new  [`MemoryUsage`] widget
    ///
    /// Arguments
    ///
    /// `attrs`: [`Attributes`] - Widget attributes which control font,
    /// foreground and background colour.
    ///
    /// `render`: [`Option<MemoryRender>`] - Optional
    /// parameter to customise the way the widget is rendered. Takes a
    /// closure that returns a String
    #[must_use]
    pub fn new(attrs: Attributes, render: Option<MemoryRender>) -> MemoryUsage {
        let memory_handle = System::new();
        MemoryUsage {
            attrs,
            render,
            memory_handle,
            update_interval: Duration::new(1, 0),
        }
    }

    fn tick(&mut self) -> Vec<Text> {
        self.memory_handle
            .refresh_memory_specifics(MemoryRefreshKind::everything());
        let used_bytes = Byte::from_u64(self.memory_handle.free_memory());
        let total_bytes = Byte::from_u64(self.memory_handle.total_memory());
        let used_swap = Byte::from_u64(self.memory_handle.used_swap());
        let total_swap = Byte::from_u64(self.memory_handle.total_swap());

        let text = if let Some(render_f) = &self.render {
            render_f.as_ref()((used_bytes, total_bytes), (used_swap, total_swap))
        } else {
            format!(
                "({used_mem}/{total_mem}) ({used_swap}/{total_swap})",
                used_mem = used_bytes.get_appropriate_unit(UnitType::Binary),
                total_mem = total_bytes.get_appropriate_unit(UnitType::Binary),
                used_swap = used_swap.get_appropriate_unit(UnitType::Binary),
                total_swap = total_swap.get_appropriate_unit(UnitType::Binary),
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

impl Widget for MemoryUsage {
    fn into_stream(mut self: Box<Self>) -> Result<WidgetStream> {
        let interval = time::interval(self.update_interval);
        let stream = IntervalStream::new(interval).map(move |_| Ok(self.tick()));

        Ok(Box::pin(stream))
    }
}
