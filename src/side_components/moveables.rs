use std::f32::consts::PI;

use lotus_script::time::delta;
use pandemist_vehicle_elements::{
    api::{key_event::KeyEventCab, vehicle_infos::a_ground},
    components::{
        general::{
            cabin_door::HandDoorWithLever,
            folding_seat::FoldingSeat,
            windows::{FoldingWindow, SlidingWindow},
        },
        gt6n::extra::Slider3d,
    },
    elements::{
        std::helper::gen_f32,
        tech::{
            key_switch::{KeyDepot, KeySwitch},
            slider::{Rollo, Slider},
        },
    },
    management::communicator::Com,
};

use crate::general::local_values::WslFlapBlocked;

pub struct Moveables {
    // A Fahrerstand
    cabin_door: HandDoorWithLever,
    key_cabin_door: KeySwitch,
    sliding_window_r: SlidingWindow,
    sliding_window_l: SlidingWindow,
    rollo: Rollo,
    rollo_r: Rollo,
    rollo_l: Rollo,
    radio_volume: Slider,
    cabin_ventilation: Slider,
    microphone: Slider3d,
    // Sliding Table
    sliding_pdedestal: Slider,
    // Rear control panel
    b_cover_flap: Slider,
    // Passenger cabin
    folding_seat_a: FoldingSeat,
    folding_seat_b: FoldingSeat,
    folding_window_a_r1: FoldingWindow,
    folding_window_a_r2: FoldingWindow,
    folding_window_c_r1: FoldingWindow,
    folding_window_c_r2: FoldingWindow,
    folding_window_b_r1: FoldingWindow,
    folding_window_b_r2: FoldingWindow,
    folding_window_b_r3: FoldingWindow,
    folding_window_b_r4: FoldingWindow,

    folding_window_a_l1: FoldingWindow,
    folding_window_a_l2: FoldingWindow,
    folding_window_a_l3: FoldingWindow,
    folding_window_c_l1: FoldingWindow,
    folding_window_c_l2: FoldingWindow,
    folding_window_c_l3: FoldingWindow,
    folding_window_c_l4: FoldingWindow,
    folding_window_b_l1: FoldingWindow,
    folding_window_b_l2: FoldingWindow,
    folding_window_b_l3: FoldingWindow,
    folding_window_b_l4: FoldingWindow,
}

impl Moveables {
    pub fn new(driver_key: KeyDepot) -> Self {
        Self {
            // A Fahrerstand
            cabin_door: HandDoorWithLever::builder(
                "AV_A_Fahrerraumtuer",
                "AV_A_Fahrerraumtuer_Riegel",
                "AV_A_Fahrerraumtuer_Klinke",
                "Fahrerraumtuer_A",
                "Fahrerraumtuer_B",
                "Fahrerraumtuer_Klinke",
                "_",
                Some(KeyEventCab::ACab),
            )
            .friction(0.01)
            .mouse_factor(1.0 / 550.0)
            .set_bolt_mode()
            .build(),

            key_cabin_door: KeySwitch::builder(
                driver_key.clone(),
                "AV_A_Fahrerraumtuer_Schluessel",
                "vis_A_Key_Fahrertuer",
                Some(KeyEventCab::ACab),
            )
            .event_turn("Key_Fahrerraumtuer")
            .event_toggle("Key_Fahrerraumtuer_Insert")
            .snd_default("Snd_A_Key_Door_Turn")
            .snd_insert("Snd_A_Key_Door_Insert")
            .snd_takeout("Snd_A_Key_Door_Takeout")
            .pullout_min()
            .max_spring()
            .build(),

            sliding_window_r: SlidingWindow::builder(
                "AV_A_Schiebefenster_R",
                "Schiebefenster_R",
                Some(KeyEventCab::ACab),
            )
            .axis_x()
            .mouse_factor(1.0 / 350.0)
            .build(),
            sliding_window_l: SlidingWindow::builder(
                "AV_A_Schiebefenster_L",
                "Schiebefenster_L",
                Some(KeyEventCab::ACab),
            )
            .axis_x()
            .mouse_factor(-1.0 / 350.0)
            .build(),
            rollo: Rollo::builder("AV_A_Rollo", "Rollo", Some(KeyEventCab::ACab))
                .mouse_factor(1.0 / 1000.0)
                .build(),
            rollo_r: Rollo::builder("AV_A_Rollo_R", "Rollo_R", Some(KeyEventCab::ACab))
                .mouse_factor(1.0 / 1000.0)
                .build(),
            rollo_l: Rollo::builder("AV_A_Rollo_L", "Rollo_L", Some(KeyEventCab::ACab))
                .mouse_factor(1.0 / 1000.0)
                .build(),
            radio_volume: Slider::builder()
                .animation("AV_A_Funklautstaerke")
                .key_event("Funklautstaerke", Some(KeyEventCab::ACab))
                .axis_x()
                .mouse_factor(1.0 / 550.0)
                .only_while_grab()
                .build(),
            cabin_ventilation: Slider::builder()
                .animation("AV_A_Belueftung")
                .key_event("Belueftungsknauf", Some(KeyEventCab::ACab))
                .axis_x()
                .mouse_factor(1.0 / 200.0)
                .only_while_grab()
                .build(),
            microphone: Slider3d::new(
                Some(KeyEventCab::ACab),
                "Mikro",
                "Mirko_Pull",
                01.0 / 450.0,
                "AV_A_Mikro_Rot_X",
                "AV_A_Mikro_Pull",
                "AV_A_Mikro_Rot_Z",
            ),
            // Schiebetisch
            sliding_pdedestal: Slider::builder()
                .animation("AV_A_Schiebetisch")
                .key_event("Schiebepodest", Some(KeyEventCab::ACab))
                .axis_x()
                .mouse_factor(-1.0 / 300.0)
                .only_while_grab()
                .build(),
            // Heck Fahrerstand
            b_cover_flap: Slider::builder()
                .animation("AV_B_Pult_Klappe")
                .key_event("PultKlappe", Some(KeyEventCab::BCab))
                .axis_y()
                .friction(10.0)
                .mouse_factor(-1.0 / 5.0)
                .lower_bumb_factor(0.0)
                .upper_bump_factor(0.5)
                .min(0.0)
                .max(170.0)
                .build(),
            // Fahrgastraum
            folding_seat_a: FoldingSeat::builder("AV_Klappsitz_A", "Klappsitz_A", None)
                .friction(0.1)
                .bump_factor(0.2)
                .mouse_factor(1.0 / 5.0)
                .spring_random(gen_f32(1.5..=3.0))
                .build(),
            folding_seat_b: FoldingSeat::builder("AV_Klappsitz_B", "Klappsitz_B", None)
                .friction(0.1)
                .bump_factor(0.2)
                .mouse_factor(1.0 / 5.0)
                .spring_random(gen_f32(1.5..=3.0))
                .build(),

            folding_window_a_r1: FoldingWindow::builder(
                "AV_Klappfenster_A_R1",
                "Klappfenster_A_R1",
                None,
            )
            .snd_open("Snd_A_WindowOpen")
            .snd_close("Snd_A_WindowClose")
            .build(),
            folding_window_a_r2: FoldingWindow::builder(
                "AV_Klappfenster_A_R2",
                "Klappfenster_A_R2",
                None,
            )
            .snd_open("Snd_A_WindowOpen")
            .snd_close("Snd_A_WindowClose")
            .build(),

            folding_window_c_r1: FoldingWindow::builder(
                "AV_Klappfenster_C_R1",
                "Klappfenster_C_R1",
                None,
            )
            .snd_open("Snd_C_WindowOpen")
            .snd_close("Snd_C_WindowClose")
            .build(),
            folding_window_c_r2: FoldingWindow::builder(
                "AV_Klappfenster_C_R2",
                "Klappfenster_C_R2",
                None,
            )
            .snd_open("Snd_C_WindowOpen")
            .snd_close("Snd_C_WindowClose")
            .build(),

            folding_window_b_r1: FoldingWindow::builder(
                "AV_Klappfenster_B_R1",
                "Klappfenster_B_R1",
                None,
            )
            .snd_open("Snd_B_WindowOpen")
            .snd_close("Snd_B_WindowClose")
            .build(),
            folding_window_b_r2: FoldingWindow::builder(
                "AV_Klappfenster_B_R2",
                "Klappfenster_B_R2",
                None,
            )
            .snd_open("Snd_B_WindowOpen")
            .snd_close("Snd_B_WindowClose")
            .build(),
            folding_window_b_r3: FoldingWindow::builder(
                "AV_Klappfenster_B_R3",
                "Klappfenster_B_R3",
                None,
            )
            .snd_open("Snd_B_WindowOpen")
            .snd_close("Snd_B_WindowClose")
            .build(),
            folding_window_b_r4: FoldingWindow::builder(
                "AV_Klappfenster_B_R4",
                "Klappfenster_B_R4",
                None,
            )
            .snd_open("Snd_B_WindowOpen")
            .snd_close("Snd_B_WindowClose")
            .build(),

            folding_window_a_l1: FoldingWindow::builder(
                "AV_Klappfenster_A_L1",
                "Klappfenster_A_L1",
                None,
            )
            .snd_open("Snd_A_WindowOpen")
            .snd_close("Snd_A_WindowClose")
            .build(),
            folding_window_a_l2: FoldingWindow::builder(
                "AV_Klappfenster_A_L2",
                "Klappfenster_A_L2",
                None,
            )
            .snd_open("Snd_A_WindowOpen")
            .snd_close("Snd_A_WindowClose")
            .build(),
            folding_window_a_l3: FoldingWindow::builder(
                "AV_Klappfenster_A_L3",
                "Klappfenster_A_L3",
                None,
            )
            .snd_open("Snd_A_WindowOpen")
            .snd_close("Snd_A_WindowClose")
            .build(),

            folding_window_c_l1: FoldingWindow::builder(
                "AV_Klappfenster_C_L1",
                "Klappfenster_C_L1",
                None,
            )
            .snd_open("Snd_C_WindowOpen")
            .snd_close("Snd_C_WindowClose")
            .build(),
            folding_window_c_l2: FoldingWindow::builder(
                "AV_Klappfenster_C_L2",
                "Klappfenster_C_L2",
                None,
            )
            .snd_open("Snd_C_WindowOpen")
            .snd_close("Snd_C_WindowClose")
            .build(),
            folding_window_c_l3: FoldingWindow::builder(
                "AV_Klappfenster_C_L3",
                "Klappfenster_C_L3",
                None,
            )
            .snd_open("Snd_C_WindowOpen")
            .snd_close("Snd_C_WindowClose")
            .build(),
            folding_window_c_l4: FoldingWindow::builder(
                "AV_Klappfenster_C_L4",
                "Klappfenster_C_L4",
                None,
            )
            .snd_open("Snd_C_WindowOpen")
            .snd_close("Snd_C_WindowClose")
            .build(),

            folding_window_b_l1: FoldingWindow::builder(
                "AV_Klappfenster_B_L1",
                "Klappfenster_B_L1",
                None,
            )
            .snd_open("Snd_B_WindowOpen")
            .snd_close("Snd_B_WindowClose")
            .build(),
            folding_window_b_l2: FoldingWindow::builder(
                "AV_Klappfenster_B_L2",
                "Klappfenster_B_L2",
                None,
            )
            .snd_open("Snd_B_WindowOpen")
            .snd_close("Snd_B_WindowClose")
            .build(),
            folding_window_b_l3: FoldingWindow::builder(
                "AV_Klappfenster_B_L3",
                "Klappfenster_B_L3",
                None,
            )
            .snd_open("Snd_B_WindowOpen")
            .snd_close("Snd_B_WindowClose")
            .build(),
            folding_window_b_l4: FoldingWindow::builder(
                "AV_Klappfenster_B_L4",
                "Klappfenster_B_L4",
                None,
            )
            .snd_open("Snd_B_WindowOpen")
            .snd_close("Snd_B_WindowClose")
            .build(),
        }
    }

    pub fn tick(&mut self, com: &mut Com) {
        //=A=Klappe==========================================

        self.key_cabin_door.tick();
        self.cabin_door.door_key_value = self.key_cabin_door.value(true) > 0;
        self.cabin_door.tick(a_ground() * 0.025 * delta());

        self.sliding_window_r.tick();
        self.sliding_window_l.tick();
        self.rollo.tick();
        self.rollo_r.tick();
        self.rollo_l.tick();
        self.radio_volume.tick();
        self.cabin_ventilation.tick();
        self.microphone.tick();

        //=A=Schiebetisch====================================

        self.sliding_pdedestal.tick();

        //=Heckpult=Klappe===================================

        let allow_flap = com.lv.get_or(WslFlapBlocked(1), false);

        if allow_flap {
            self.b_cover_flap.lower_bump_factor = 0.4;
            self.b_cover_flap.min = 7.8;
        } else {
            self.b_cover_flap.lower_bump_factor = -1.0;
            self.b_cover_flap.min = 0.0;
        }
        self.b_cover_flap.force = -600.0 * (self.b_cover_flap.pos * PI / 180.0).cos();
        self.b_cover_flap.tick();

        //=Fahrgastraum======================================

        self.folding_seat_a.tick();
        self.folding_seat_b.tick();

        self.folding_window_a_r1.tick();
        self.folding_window_a_r2.tick();
        self.folding_window_c_r1.tick();
        self.folding_window_c_r2.tick();
        self.folding_window_b_r1.tick();
        self.folding_window_b_r2.tick();
        self.folding_window_b_r3.tick();
        self.folding_window_b_r4.tick();
        self.folding_window_a_l1.tick();
        self.folding_window_a_l2.tick();
        self.folding_window_a_l3.tick();
        self.folding_window_c_l1.tick();
        self.folding_window_c_l2.tick();
        self.folding_window_c_l3.tick();
        self.folding_window_c_l4.tick();
        self.folding_window_b_l1.tick();
        self.folding_window_b_l2.tick();
        self.folding_window_b_l3.tick();
        self.folding_window_b_l4.tick();

        //===================================================
    }
}
