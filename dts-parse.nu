def dts-parse [] {
    (cat                                   
    "data/linux/arch/arm/boot/dts/sama5d2.dtsi"
    "data/linux/arch/arm/boot/dts/at91-sama5d27_wlsom1.dtsi"
    "data/linux/arch/arm/boot/dts/at91-sama5d27_wlsom1_ek.dts"
    | ./dts-to-json | save --force ek.json )
}
