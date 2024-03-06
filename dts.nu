const dtc_source = "arch/arm/boot/dts/at91-sama5d27_wlsom1_ek.dts"
const dtc_root = "~/projects/microchip/linux"

export def dtc-run [source: string=$dtc_source, root: string=$dtc_root] {
    dtc-expand $source $root | dtc -I dts -O dts -s
}

export def dtc-expand [source: string=$dtc_source, root: string=$dtc_root] {
    cd $root
    (cpp 
        -nostdinc                                  
	    -Iscripts/dtc/include-prefixes  
	    -undef 
        -D__DTS__ 
        -xassembler-with-cpp 
        $source)
}

export def dts-parse [] {
    (cat                                   
    "data/linux/arch/arm/boot/dts/sama5d2.dtsi"
    "data/linux/arch/arm/boot/dts/at91-sama5d27_wlsom1.dtsi"
    "data/linux/arch/arm/boot/dts/at91-sama5d27_wlsom1_ek.dts"
    | ./dts-to-json | save --force ek.json )
}
