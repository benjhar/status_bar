use anyhow::Result;
use byte_unit::{Byte, ByteUnit};
use cnx::text::{Attributes, Color, Font, Padding, PagerAttributes};
use cnx::widgets::ActiveWindowTitle;
use cnx::{widgets, Cnx, Position};
use cnx_contrib::widgets::{battery, cpu, volume};
use status_bar::memory;
use sysinfo::{System, SystemExt};

const DEFAULT_FONT: &str = "monospace";

fn workspace_widget() -> widgets::Pager {
    let focused_workspace_attrs = Attributes {
        font: Font::new(DEFAULT_FONT),
        fg_color: Color::white(),
        bg_color: Some(Color::from_rgb(20, 76, 166)),
        padding: Padding::new(8.0, 8.0, 0.0, 0.0),
    };

    let busy_workspace_attrs = Attributes {
        font: Font::new(DEFAULT_FONT),
        fg_color: Color::white(),
        bg_color: Some(Color::from_rgb(100, 100, 100)),
        padding: Padding::new(8.0, 8.0, 0.0, 0.0),
    };

    let empty_workspace_attrs = Attributes {
        font: Font::new(DEFAULT_FONT),
        fg_color: Color::from_rgb(100, 100, 100),
        bg_color: None,
        padding: Padding::new(8.0, 8.0, 0.0, 0.0),
    };

    let pager_attrs = PagerAttributes {
        active_attr: focused_workspace_attrs,
        inactive_attr: empty_workspace_attrs,
        non_empty_attr: busy_workspace_attrs,
    };

    widgets::Pager::new(pager_attrs)
}

fn window_title_widget() -> ActiveWindowTitle {
    let window_title_attrs = Attributes {
        font: Font::new(DEFAULT_FONT),
        fg_color: Color::white(),
        bg_color: None,
        padding: Padding::new(5.0, 5.0, 0.0, 0.0),
    };

    ActiveWindowTitle::new(window_title_attrs)
}

fn battery_widget() -> battery::Battery {
    let battery_attrs = Attributes {
        font: Font::new(DEFAULT_FONT),
        fg_color: Color::white(),
        bg_color: None,
        padding: Padding::new(5.0, 5.0, 0.0, 0.0),
    };

    let render = Box::new(|battery_info: battery::BatteryInfo| {
        let charge = battery_info.capacity;
        let mut colour = Color::white().to_hex();
        if charge > 80 {
            colour = Color::green().to_hex();
        }
        let mut emoji = match battery_info.status {
            battery::Status::Charging => "🔌",
            _ => "🔋",
        };

        if charge < 20 {
            colour = Color::red().to_hex();
            emoji = "🪫";
        }

        format!(
            "<span foreground=\"#808080\">[</span>{emoji}<span foreground=\"{colour}\">{charge}%</span><span foreground=\"#808080\">]</span>"
        )
    });

    battery::Battery::new(
        battery_attrs,
        Color::from_rgb(191, 2, 2),
        None,
        Some(render),
    )
}

fn cpu_widget() -> Result<cpu::Cpu> {
    let cpu_attrs = Attributes {
        font: Font::new(DEFAULT_FONT),
        fg_color: Color::white(),
        bg_color: None,
        padding: Padding::new(5.0, 5.0, 0.0, 0.0),
    };

    let render = Box::new(|load| {
        let mut color = Color::yellow().to_hex();
        if load < 5 {
            color = Color::green().to_hex();
        }
        if load > 50 {
            color = Color::red().to_hex();
        }
        format!(
            "<span foreground=\"#808080\">[</span>⚡<span foreground=\"{color}\">{load}%</span><span foreground=\"#808080\">]</span>"
        )
    });

    cpu::Cpu::new(cpu_attrs, Some(render))
}

fn memory_usage_widget() -> memory::MemoryUsage {
    let memory_attrs = Attributes {
        font: Font::new(DEFAULT_FONT),
        fg_color: Color::white(),
        bg_color: None,
        padding: Padding::new(5.0, 5.0, 0.0, 0.0),
    };

    let render = Box::new(|memory_handle: &System| {
        let total_memory = Byte::from_bytes(memory_handle.total_memory().into());
        let used_memory = Byte::from_bytes(memory_handle.free_memory().into());
        let total_swap_memory = Byte::from_bytes(memory_handle.total_swap().into());
        let used_swap_memory = Byte::from_bytes(memory_handle.free_swap().into());

        let mut mem_colour = Color::white().to_hex();

        if used_memory.get_bytes() >= total_memory.get_bytes() / 2 {
            mem_colour = Color::yellow().to_hex();
        }
        if used_memory.get_bytes() >= total_memory.get_bytes() / 5 * 4 {
            mem_colour = Color::red().to_hex();
        }

        let mut swap_colour = Color::white().to_hex();

        if used_swap_memory.get_bytes() >= total_swap_memory.get_bytes() / 2 {
            swap_colour = Color::yellow().to_hex();
        }
        if used_swap_memory.get_bytes() >= total_swap_memory.get_bytes() / 5 * 4 {
            swap_colour = Color::red().to_hex();
        }

        let used_mem = used_memory.get_adjusted_unit(ByteUnit::GB).get_value();
        let total_mem = total_memory.get_adjusted_unit(ByteUnit::GB).format(1);
        let used_swap = used_swap_memory.get_adjusted_unit(ByteUnit::GB).get_value();
        let total_swap = total_swap_memory.get_adjusted_unit(ByteUnit::GB).format(1);

        format!("<span foreground=\"#808080\">[</span>🧠 <span foreground=\"{mem_colour}\">{used_mem:.1}</span>/{total_mem}<span foreground=\"#808080\">]</span> <span foreground=\"#808080\">[</span>💾 <span foreground=\"{swap_colour}\">{used_swap:.1}</span>/{total_swap}<span foreground=\"#808080\">]</span>")
    });

    memory::MemoryUsage::new(memory_attrs, Some(render))
}

fn volume_widget() -> volume::Volume {
    let volume_attrs = Attributes {
        font: Font::new(DEFAULT_FONT),
        fg_color: Color::white(),
        bg_color: None,
        padding: Padding::new(5.0, 5.0, 0.0, 0.0),
    };

    volume::Volume::new(volume_attrs)
}

fn clock_widget() -> widgets::Clock {
    let clock_attributes = Attributes {
        font: Font::new(DEFAULT_FONT),
        fg_color: Color::white(),
        bg_color: None,
        padding: Padding::new(5.0, 5.0, 0.0, 0.0),
    };

    widgets::Clock::new(
        clock_attributes.clone(),
        Some("%H:%M %a %d-%m-%Y".to_string()),
    )
}

fn main() -> Result<()> {
    let mut bar = Cnx::new(Position::Top);

    bar.add_widget(workspace_widget());
    bar.add_widget(window_title_widget());
    bar.add_widget(battery_widget());

    if let Ok(cpu_w) = cpu_widget() {
        bar.add_widget(cpu_w)
    };

    bar.add_widget(memory_usage_widget());
    bar.add_widget(volume_widget());
    bar.add_widget(clock_widget());

    bar.run()?;
    Ok(())
}
