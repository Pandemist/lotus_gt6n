use lotus_extra::messages::std_helper::send_veh_number;
use lotus_script::time::game_time;
use lotus_script::{content::ContentId, rand::gen_u64};
use pandemist_vehicle_elements::api::key_event::KeyEventCab;
use pandemist_vehicle_elements::api::variable::{get_var, set_var};
use pandemist_vehicle_elements::api::vehicle_infos::{set_veh_number, veh_number};
use pandemist_vehicle_elements::elements::std::ad_ids::get_horizontal_ad;
use pandemist_vehicle_elements::elements::std::helper::{enhance_string, get_random_element};
use pandemist_vehicle_elements::elements::tech::switches::Switch;
use pandemist_vehicle_elements::management::communicator::Com;
use pandemist_vehicle_elements::messages::pandemist_messages::send_gpm_state;
use time::{Date, PrimitiveDateTime, Time};

//+++++++++++++++++++++++++++++++++++++++++++++++++++

//+++++++++++++++++++++++++++++++++++++++++++++++++++

pub fn determ_veh_number() {
    let gto_numbers: Vec<u64> = vec![
        1201, 1204, 1205, 1225, 1226, 1228, 1229, 1231, 1232, 1233, 1236, 1237, 1238, 1239, 1240,
        1245, 1249, 1250, 1251, 1253, 1254, 1255, 1256, 1257, 1260, 1261, 1262, 1263,
    ];

    let gtu_numbers: Vec<u64> = vec![
        1502, 1503, 1506, 1507, 1508, 1509, 1510, 1511, 1512, 1513, 1514, 1515, 1516, 1517, 1518,
        1519, 1520, 1521, 1522, 1523, 1524, 1527, 1530, 1534, 1535, 1541, 1542, 1543, 1544, 1546,
        1547, 1548, 1552, 1558, 1559, 1564, 1565, 1566, 1567, 1568, 1569, 1570, 1571, 1572, 1573,
        1575, 1576, 1577, 1578, 1579, 1580, 1581, 1582, 1583, 1584, 1585, 1586, 1587, 1588, 1589,
        1590, 1591, 1592, 1593, 1594, 1595, 1596, 1597, 1598, 1599, 1600, 1601, 1602, 1603, 1604,
        1605,
    ];

    let vehicle_variant = const_veh_variant();

    let is_gt6 = vehicle_variant == FahrzeugVariante::Gt6n;
    let is_gto = vehicle_variant == FahrzeugVariante::Gt6o;
    let mut veh_number = veh_number();

    if veh_number.is_empty() {
        let new_veh_number = if is_gt6 {
            gen_u64(1000..=1105)
        } else if is_gto {
            *get_random_element(&gto_numbers)
        } else {
            *get_random_element(&gtu_numbers)
        };

        veh_number = enhance_string(new_veh_number.to_string(), 4, '0');
        set_veh_number(veh_number.clone());
    }
    send_veh_number(veh_number.clone());
}

pub fn textures() {
    set_var(
        "tex_Fahrerraum_1",
        ContentId {
            user_id: 1000,
            sub_id: 1696752439,
        },
    );

    set_var(
        "tex_Fahrerraum_1_night",
        ContentId {
            user_id: 1000,
            sub_id: 1534618898,
        },
    );

    set_var(
        "tex_Fahrerraum_2",
        ContentId {
            user_id: 1000,
            sub_id: 1314997822,
        },
    );

    set_var(
        "tex_Label_1",
        ContentId {
            user_id: 1000,
            sub_id: 1901837693,
        },
    );

    set_var(
        "tex_Zusatz_1",
        ContentId {
            user_id: 1000,
            sub_id: 1866265699,
        },
    );

    let is_gtu = get_var::<bool>("isGTU");

    if is_gtu {
        set_var(
            "tex_Fahrerraum_1",
            ContentId {
                user_id: 1000,
                sub_id: 278925710,
            },
        );

        set_var(
            "tex_Fahrerraum_1_night",
            ContentId {
                user_id: 1000,
                sub_id: 49203581,
            },
        );

        set_var(
            "tex_Fahrerraum_2",
            ContentId {
                user_id: 1000,
                sub_id: 371204603,
            },
        );

        set_var(
            "tex_Label_1",
            ContentId {
                user_id: 1000,
                sub_id: 524103179,
            },
        );

        set_var(
            "tex_Zusatz_1",
            ContentId {
                user_id: 1000,
                sub_id: 957218721,
            },
        );
    }
}

pub fn set_fuse(com: &mut Com) {
    com.fuse.register(
        "NotbremseLoesen",
        Switch::builder("AV_A_Kss_Notbremse_Loesen", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Notbremse_loesen")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "Schienenbremsschuetz",
        Switch::builder("AV_A_Kss_Schienenbremsschuetz", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Schienenbremsschuetz")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "Stromabnehmerantrieb",
        Switch::builder("AV_A_Kss_Stromabnehmerantrieb", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Stromabnehmerantrieb")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "Hauptschalterantrieb1",
        Switch::builder("AV_A_Kss_Hauptschalter_Antrieb_1", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Hauptschalterantrieb_1")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "Hauptschalterantrieb2",
        Switch::builder("AV_A_Kss_Hauptschalter_Antrieb_2", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Hauptschalterantrieb_2")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "IKEELAversorgung",
        Switch::builder("AV_A_Kss_IKE_ELA_Versorgung", Some(KeyEventCab::ACab))
            .event_toggle("Kss_IKE_ELA_Versorgung")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "IKEaufruesten",
        Switch::builder("AV_A_Kss_IKE_Aufruestungssignal", Some(KeyEventCab::ACab))
            .event_toggle("Kss_IKE_Aufruestung")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "FunkIMUVersorgung",
        Switch::builder("AV_A_Kss_Funk_IMU_Versorgung", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Funk_IMU_Versorgung")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "Notruf",
        Switch::builder("AV_A_Kss_Notruf", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Notruf")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "KWRsifaEingangssingal",
        Switch::builder("AV_A_Kss_KWR_Sifa_Eingangssignal", Some(KeyEventCab::ACab))
            .event_toggle("Kss_KWR_Sifa_Eingangssignale")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "SPSversorgung",
        Switch::builder("AV_A_Kss_SPS_Versorgung", Some(KeyEventCab::ACab))
            .event_toggle("Kss_SPS_Versorgung")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "Beleuchtungssteuerung",
        Switch::builder("AV_A_Kss_Beleuchtungssteuerung", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Beleuchtungssteuerung")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "Begrenzungslicht",
        Switch::builder("AV_A_Kss_Begrenzungslicht", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Begrenzungslicht")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "NahFernlicht",
        Switch::builder("AV_A_Kss_Nah_Fernlicht", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Nah_Fernlicht")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "BlinkerLinks",
        Switch::builder("AV_A_Kss_Blinkerrelais_Links", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Blinkerrelais_Links")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "BlinkerRechts",
        Switch::builder("AV_A_Kss_Blinkerrelais_Rechts", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Blinkerrelais_Rechts")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "Sandrohrheizung",
        Switch::builder("AV_A_Kss_Sandrohrheizung", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Sandstreuerheizung")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "Fahrersitz",
        Switch::builder("AV_A_Kss_Fahrersitz", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Fahrersitz")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "Frontscheibenheizung",
        Switch::builder("AV_A_Kss_Frontscheibenheizung", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Frontscheibenheizung")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "Seitenscheibenheizung",
        Switch::builder("AV_A_Kss_Seitenscheibenheizung", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Seitenscheibenheizung")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "SeitenscheibenheizungSteuerung",
        Switch::builder(
            "AV_A_Kss_Seitenscheibenheizung_Steuerung",
            Some(KeyEventCab::ACab),
        )
        .event_toggle("Kss_Seitenscheibenheizung_Steuerung")
        .snd_plus("Snd_A_Kss_On")
        .snd_minus("Snd_A_Kss_Off")
        .init(true)
        .build(),
    );
    com.fuse.register(
        "Spurkranzschmierung",
        Switch::builder("AV_A_Kss_Spurkranzschmierung", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Spurkranzschmierung")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "VersorgungTuer1",
        Switch::builder("AV_A_Kss_Versorgung_Tuer_1", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Versorgung_Tuer_1")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "SteuerungTuer1",
        Switch::builder("AV_A_Kss_Steuerung_Tuer_1", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Steuerung_Tuer_1")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "ZentraleTuersteuerung",
        Switch::builder("AV_A_Kss_Zentrale_Tuersteuerung", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Zentrale_Tuersteuerung")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "HubliftLeistungsteil",
        Switch::builder("AV_A_Kss_Hublift_Leistungsteil", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Hublift_Leistungsteil")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "HubliftSteuersignal",
        Switch::builder("AV_A_Kss_Hublift_Steuerung", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Hublift_Steuersignal")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "HubliftKompressor",
        Switch::builder("AV_A_Kss_Hublift_Kompressor", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Hublift_Kompressor")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "HauptschalterTW1",
        Switch::builder("AV_A_Kss_Hauptschalter_TW1", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Hauptschalter_TW1")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "HauptschalterTW2",
        Switch::builder("AV_A_Kss_Hauptschalter_TW2", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Hauptschalter_TW2")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
    com.fuse.register(
        "BordnetzumrichterDBU2",
        Switch::builder("AV_A_Kss_Bordnetzumrichter_UBU_2", Some(KeyEventCab::ACab))
            .event_toggle("Kss_Bordnetzumrichter_UBU_2")
            .snd_plus("Snd_A_Kss_On")
            .snd_minus("Snd_A_Kss_Off")
            .init(true)
            .build(),
    );
}

pub fn initialize(com: &mut Com) {
    set_fuse(com);

    // Const Vehicle number
    set_var(
        "vis_Wagennummer",
        !get_var::<bool>("const_Wagennummernanzeige"),
    );

    // Const Zustand
    let vehicle_variant = const_veh_variant();

    let variant_gt6n = vehicle_variant == FahrzeugVariante::Gt6n;
    let variant_gt6o = vehicle_variant == FahrzeugVariante::Gt6o;
    let variant_gt6u = vehicle_variant == FahrzeugVariante::Gt6u;

    set_var("isGT6", variant_gt6n);
    set_var("isGTO", variant_gt6o);
    set_var("isGTU", variant_gt6u);
    set_var("isNotGTU", !variant_gt6u);

    determ_veh_number();

    send_gpm_state();

    // Haltestangen setzten
    let veh_number_int = get_var::<u32>("veh_number_int");
    let old_hst_1 = variant_gt6n && veh_number_int <= 1060;
    let old_hst_2 = variant_gt6o && veh_number_int <= 1260;
    let old_hst_3 = variant_gt6u && veh_number_int <= 1560;
    let old_handrails = old_hst_1 || old_hst_2 || old_hst_3;

    set_var("const_AlteHaltestangen", old_handrails);

    set_var("vis_HaltestangenAlt", old_handrails);
    set_var("vis_HaltestangenNeu", !old_handrails);

    set_var("vis_HaltestangenAlt_GTO", old_handrails);
    set_var("vis_HaltestangenNeu_GTO", !old_handrails);

    set_var("vis_HaltestangenAlt_GTU", false);
    set_var("vis_HaltestangenNeu_GTU", false);

    // Const 750V Aufkleber
    if past_date_750v() {
        set_var("vis_750V", !variant_gt6u);
        set_var("vis_750V_GTU", variant_gt6u);
    }

    // Const Spurweite
    let const_track_gauge = get_var::<i32>("const_Spurweite");

    set_var("vis_Spurweite_Normal", const_track_gauge == 0);
    set_var("vis_Spurweite_Schmalspur", const_track_gauge == 1);
    set_var("vis_Spurweite_Kapspur", const_track_gauge == 2);
    set_var("vis_Spurweite_Meter", const_track_gauge == 3);

    // Const WLan Antenne
    set_var("vis_WLAN_Dongle", get_var::<bool>("const_WLAN_Antenne"));

    // Nicht an konstanten gebundene Sichtbarkeiten
    set_var("vis_OldDisplays", true);
    set_var("vis_NewDisplays", false);

    set_var("const_LEDLampe", false);
    set_var("const_Blinker", false);

    set_var(
        "tex_Werbetafel_1",
        ContentId {
            user_id: 5749281,
            sub_id: 218749186,
        },
    );

    set_var("tex_Werbetafel_2", get_horizontal_ad());

    textures();
}

#[derive(Debug, PartialEq, Eq)]
pub enum FahrzeugVariante {
    Gt6n,
    Gt6o,
    Gt6u,
}

pub fn const_veh_variant() -> FahrzeugVariante {
    match get_var::<i32>("const_Zustand") {
        2 => FahrzeugVariante::Gt6u,
        1 => FahrzeugVariante::Gt6o,
        _ => FahrzeugVariante::Gt6n,
    }
}

pub fn past_date_750v() -> bool {
    let dt = game_time().primitive_date_time();

    let ref_dt = PrimitiveDateTime::new(
        Date::from_calendar_date(2023, time::Month::April, 2).unwrap(),
        Time::from_hms(1, 30, 0).unwrap(),
    );

    dt > ref_dt
}
