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

export def "survey pcbs" [tree] {
    let ki_parse = $env.kiops_bin | path join ki_parse
    (ls data/kicad/**/*.kicad_pcb
        | insert footprints { |p| open --raw $p.name | ^$ki_parse footprints | from json}
        | select name footprints
        | flatten -a
    )
}

export def "survey symbols" [] {
    let ki_parse = $env.kiops_bin | path join ki_parse
    (ls data/kicad/**/*.kicad_sch
        | insert symbols { |p| open --raw $p.name | ^$ki_parse symbols | from json}
        | select name symbols
        | flatten -a
    )
}

export def "survey sheets" [] {
    let ki_parse = $env.kiops_bin | path join ki_parse
    (ls data/kicad/**/*.kicad_sch
        | insert sheets { |p| open --raw $p.name | ^$ki_parse sheets | from json}
        | select name sheets
        | flatten -a
    )
}
