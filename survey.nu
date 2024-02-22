
def "survey fp" [] {
    (ls data/kicad/**/*.kicad_mod 
        | insert hash {|p| open --raw $p.name | hash md5 } 
        | insert dirname {|p| $p.name | path dirname }  
        | insert basename {|p| $p.name | path basename }  
        | sort-by hash 
        | select hash basename dirname
        | save --force survey.csv )
}

def "survey pcb" [] {
    (ls data/kicad/**/*.kicad_pcb
        | insert footprints { |p| open --raw $p.name | ./ki_parse footprints }
        | select name footprints
    )
}


def "survey sch" [] {
    (ls data/kicad/**/*.kicad_sch
        | insert symbols { |p| open --raw $p.name | ./ki_parse symbols }
        | select name symbols
    )
}