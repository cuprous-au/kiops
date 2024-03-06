def "format all" [where: string] {
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

def "hash fps" [] {
    (ls data/kicad/**/*.kicad_mod 
        | insert hash {|p| open --raw $p.name | hash md5 } 
        | insert dirname {|p| $p.name | path dirname }  
        | insert basename {|p| $p.name | path basename }  
        | sort-by hash 
        | select hash basename dirname
        | save --force survey.csv )
}
def "find symlibs" [library: path ] {
    glob ($library | path join **/*.kicad_sym) | path relative-to $library | with-names
}

def "find fplibs" [library: path] {
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

def "format table" [kind: string, library_relative: string] {
    let entries = each { |e| 
        ('(lib (name "' + 
        $e.name + 
        '")(type "KiCad")(uri "' + 
        ( '${KIPRJMOD}' | path join $library_relative $e.location )  + 
        '")(options "")(descr ""))')
    } 
    [('(' + $kind) '(version 7)'] ++ $entries ++ [')'] | str join "\n"
}

def "make tables" [project: path library_relative: string ="../library"] {
    let  library = $project | path join $library_relative | path expand
    (find symlibs $library | 
        format table sym_lib_table $library_relative | 
        save -f ($project | path join 'sym-lib-table'))
    (find fplibs $library | 
        format table fp_lib_table $library_relative | 
        save -f ($project | path join 'fp-lib-table'))
}

