export-env { 
    $env.kiops_lib_location = ($env.FILE_PWD | path join "collected") 
    $env.kiops_bin = ($env.FILE_PWD | path join "target" "release") 
    $env.kicad_cli = "/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli" 
}

export def ki [subject: string verb: string object: string] {
    ^$env.kicad_cli $subject $verb $object
}

export def "upgrade footprints" [] {
    let dir = $env.kiops_lib_location | path join "Cuprous.pretty"
    let result = (ki fp upgrade $dir)
    [$dir $result]
}

export def "upgrade symlibs" [] {
    let dir = $env.kiops_lib_location | path join "symlibs"
    ls ($dir | path join *.kicad_sym) | each { |p|  
        let result = (ki sym upgrade $p.name) 
        [$p.name $result]
    }
}

export def "merge symlibs" [] {
    let ki_merge = $env.kiops_bin | path join ki_merge
    let output = $env.kiops_lib_location | path join "Cuprous.kicad_sym"
    let input = $env.kiops_lib_location | path join "symlibs"
    let symlibs = ls ($input | path join *.kicad_sym) 
    let accum = open --raw ($symlibs | get 0.name)
    let merged = $symlibs | skip 1 | reduce --fold $accum { |p, accum|  
        $accum | ^$ki_merge $p.name 
    }
    $merged | save --raw --force $output
}

export def "install libs" [dest: string] {
    if ($dest | path type) != dir {
        return "destination does not exist or not a directory"
    } 
    let fplib = $dest | path join Cuprous.pretty
    mkdir $fplib
    ls ($env.kiops_lib_location | path join "Cuprous.pretty" "*") | each {|p| cp $p.name $fplib}
    [Cuprous.kicad_sym fp-lib-table sym-lib-table] | each { |name|
        cp ($env.kiops_lib_location | path join $name) ($dest | path join $name) 
    }
}

export def "survey pcbs" [] {
    let ki_parse = $env.kiops_bin | path join ki_parse
    (ls **/*.kicad_pcb
        | insert footprints { |p| open --raw $p.name | ^$ki_parse footprints | from json}
        | select name footprints
        | flatten -a
    )
}

export def "survey symbols" [] {
    let ki_parse = $env.kiops_bin | path join ki_parse
    (ls **/*.kicad_sch
        | insert symbols { |p| open --raw $p.name | ^$ki_parse symbols | from json}
        | select name symbols
        | flatten -a
    )
}

export def "survey sheets" [] {
    let ki_parse = $env.kiops_bin | path join ki_parse
    (ls **/*.kicad_sch
        | insert sheets { |p| open --raw $p.name | ^$ki_parse sheets | from json}
        | select name sheets
        | flatten -a
    )
}

# Generate fabrication files from a KiCAD project.
export def "fabricate" [
    projdir: path # The directory containing the KiCAD project
    dest: string = "plot" # The directory for the output, relative to projdir
] {
    cd $projdir
    let input = glob *.kicad_pcb | first
    let stem = ($input | path parse).stem
    let output = $stem ++ "-" ++ (date now |  format date %F) ++ ".zip"

    rm -rf $dest
    mkdir $dest
    
    ^$env.kicad_cli pcb export drill $input --output ($dest | path join "")
    ^$env.kicad_cli pcb export gerbers $input --output ($dest | path join "")
    ^$env.kicad_cli pcb export pos $input --output ($dest | path join ($stem ++ ".pos"))
    create bom . | save ($dest | path join ($stem ++ "-bom.csv"))
    create bom-grouped . | save ($dest | path join ($stem ++ "-grouped-bom.csv"))

    rm -f $output
    ^zip -r  $output $dest
}

export def "create bom" [projdir: string] {
    cd $projdir
    let ki_parse = $env.kiops_bin | path join ki_parse
    (glob *.kicad_sch 
        | each { |s| open $s | ^$ki_parse symbols  | from json } 
        | flatten 
        | where lib_id != "Connector:TestPoint" and unit == 1
        | each { |r| if $r.dnp != "yes" and $r.MPN? == null {print -e ("Missing MPN for " ++ $r.reference)}; $r }
        | update dnp { |r| if $r.dnp == "yes" {"DNP"} else {""}}
        | sort-by --natural reference
        | select reference manufacturer? MPN? value description? footprint? dnp supply?)
}

export def "create bom-grouped" [projdir: string] {
    def gather [] {uniq | str join " "}
    (create bom $projdir 
        | where dnp != "DNP" 
        | group-by --to-table MPN 
        | each { |r|   
            let items = $r.items
            
            { 
                refs: ($items.reference | gather)
                manufacturer: ($items.manufacturer | gather)
                MPN: $r.group
                value: ($items.value | gather)
                description: ($items.description | gather)
                footprint: ($items.footprint | gather)
                count: ($items | length)
                supply: ($items.supply | gather)
            }
        })
}

export def "render step" [projdir: string] {
    cd $projdir
    let input = glob *.kicad_pcb | first
    ^$env.kicad_cli pcb export step --subst-models --force $input
}


export def "render pdf" [projdir: string] {
    cd $projdir
    let project = glob *.kicad_pro | first
    let input = $project | path parse | get stem | append "kicad_sch" | str join "." 
    ^$env.kicad_cli sch export pdf $input
}
