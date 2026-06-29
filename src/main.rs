#![no_std]
#![no_main]

use embassy_rp::{
    bind_interrupts,
    peripherals::{DMA_CH0, PIN_5, PIO0},
    pio::{Pio, PioPin},
    dma::Channel,
};
use embassy_time::Timer;
use rmk::macros::rmk_keyboard;
use rmk::event::ControllerEvent;
use ws2812_pio::Ws2812;
use rp2040_hal::clocks::init_clocks_and_plls;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>;
});

pub struct LedController {
    ws2812: Ws2812<embassy_rp::pio::Pio<PIO0>, PioPin<5>>,
    led_count: usize,
    caps_lock: bool,
    current_layer: u8,
}

impl LedController {
    pub fn new(
        ws2812: Ws2812<embassy_rp::pio::Pio<PIO0>, PioPin<5>>,
        led_count: usize,
    ) -> Self {
        Self {
            ws2812,
            led_count,
            caps_lock: false,
            current_layer: 0,
        }
    }

    fn update_leds(&mut self) {
        let mut colors = vec![[0, 0, 0]; self.led_count];

        // 灯0: CapsLock 指示 (绿色)
        if self.caps_lock {
            colors[0] = [0, 255, 0];
        }

        // 灯1: 层状态指示
        match self.current_layer {
            0 => colors[1] = [0, 0, 0],       // 默认层: 灭
            1 => colors[1] = [255, 0, 0],     // 层1: 红色
            2 => colors[1] = [0, 0, 255],     // 层2: 蓝色
            3 => colors[1] = [255, 255, 0],   // 层3: 黄色
            _ => colors[1] = [255, 0, 255],   // 其他: 紫色
        }

        let _ = self.ws2812.write(colors.as_slice());
    }

    async fn on_led_indicator_event(&mut self, event: ControllerEvent) {
        if let ControllerEvent::KeyboardIndicator(state) = event {
            self.caps_lock = state.caps_lock;
            self.update_leds();
        }
    }

    async fn on_layer_change_event(&mut self, event: ControllerEvent) {
        if let ControllerEvent::LayerChange(layer) = event {
            self.current_layer = layer;
            self.update_leds();
        }
    }
}

#[rmk_keyboard]
mod keyboard {
    use super::*;

    #[controller(event)]
    fn led_controller(p: &mut embassy_rp::Peripherals) -> LedController {
        let pio = Pio::new(p.PIO0, Irqs);
        let (pio, sm0, _, _, _) = pio.split();

        let ws2812 = Ws2812::new(
            pio,
            sm0,
            p.DMA_CH0,
            p.PIN_5.degrade(),
        );

        LedController::new(ws2812, 2)
    }
}