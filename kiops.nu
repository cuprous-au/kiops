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
