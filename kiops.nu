def "ki symbols" [library: path ] {
    glob ($library | path join **/*.kicad_sym) | path relative-to $library | with-names
}

def "ki footprints" [library: path] {
    glob ($library | path join **/*.kicad_mod) | path relative-to $library | path dirname | uniq | with-names
}

def with-names [] {
    collect { |paths| 
        let names = $paths | path parse | get stem
        if ($names | uniq --repeated | is-empty) {
            $names | wrap name | merge ($paths | wrap location)
        } else {
            print -e 'Error: duplicate library name found'
        }
    }
}

def "ki table-format" [kind: string, library_relative: string] {
    let entries = each { |e| 
        ('(lib (name "' + 
        $e.name + 
        '")(type "KiCad")(uri "' + 
        ( '${KIPRJMOD}' | path join $library_relative $e.location )  + 
        '")(options "")(descr ""))')
    } 
    [('(' + $kind) '(version 7)'] ++ $entries ++ [')'] | str join "\n"
}

def "ki tables" [project: path library_relative: string ="../library"] {
    let  library = $project | path join $library_relative | path expand
    (ki symbols $library | 
        ki table-format sym_lib_table $library_relative | 
        save -f ($project | path join 'sym-lib-table'))
    (ki footprints $library | 
        ki table-format fp_lib_table $library_relative | 
        save -f ($project | path join 'fp-lib-table'))
}

def "survey fp" [] {
    (ls data/kicad/**/*.kicad_mod 
        | insert hash {|p| open --raw $p.name | hash md5 } 
        | insert dirname {|p| $p.name | path dirname }  
        | insert basename {|p| $p.name | path basename }  
        | sort-by hash 
        | select hash basename dirname
        | save --force survey.csv )
}

def "collect footprints" [] {
    let dir = "./collected/Cuprous.pretty"
    mkdir $dir
    (ls data/kicad/**/*.kicad_mod | each { |p|
        let basename = $p.name | path basename
        let target = $dir | path join $basename
        let exists = $target | path exists
        if not $exists {
            cp $p.name $target
        }
        [$basename $exists]
    })
}

def "collect symlibs" [] {
    let dir = "./collected/symlibs"
    mkdir $dir
    (ls data/kicad/**/*.kicad_sym | each { |p|
        let basename = $p.name | path basename
        let target = $dir | path join $basename
        let exists = $target | path exists
        if not $exists {
            cp $p.name $target
        }
        [$basename $exists]
    })
}

def ki [subject: string verb: string object: string] {
    /Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli $subject $verb $object
}

def "upgrade footprints" [] {
    let dir = "./collected/Cuprous.pretty"
    let result = (ki fp upgrade $dir)
    [$dir $result]
}

def "upgrade symlibs" [] {
    let dir = "./collected/symlibs"
    ls ($dir | path join *.kicad_sym) | each { |p|  
        let result = (ki sym upgrade $p.name) 
        [$p.name $result]
    }
}

def "merge symlibs" [] {
    let output = "./collected/Cuprous.kicad_sym"
    let input = "./collected/symlibs"
    let symlibs = ls ($input | path join *.kicad_sym) 
    let accum = open --raw ($symlibs | get 0.name)
    let merged = $symlibs | skip 1 | reduce --fold $accum { |p, accum|  
        $accum | ./ki_merge $p.name 
    }
    $merged | save --raw --force $output
}

def "collect all" [where: string] {
    mkdir $where
    (ls data/kicad/**/*.kicad_sch | append (ls data/kicad/**/*.kicad_pcb) | each { |p|
        let basename = ($p.name | path basename)
        let target = (echo $where | path join $basename)
        if not ($target | path exists) {
            open --raw $p.name | ./ki_parse format -s | save --raw $target
            $basename
        }
    })
}

def "survey pcbs" [] {
    (ls data/kicad/**/*.kicad_pcb
        | insert footprints { |p| open --raw $p.name | ./ki_parse footprints | from json}
        | select name footprints
        | flatten -a
    )
}

def "survey symbols" [] {
    (ls data/kicad/**/*.kicad_sch
        | insert symbols { |p| open --raw $p.name | ./ki_parse symbols | from json}
        | select name symbols
        | flatten -a
    )
}

def "survey sheets" [] {
    (ls data/kicad/**/*.kicad_sch
        | insert sheets { |p| open --raw $p.name | ./ki_parse sheets | from json}
        | select name sheets
        | flatten -a
    )
}
