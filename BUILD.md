# Building

First build the rust code:

```
cargo build --release
```

Next import the nushell module:

```
$ nu
> use kiops.nu
```

Now adjust environment variables if necessary. To set the location of the KiCAD CLI (the default is shown here):

```
> $env.kicad_cli = "/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli" 
```

That is enough for the `kiops fabricate` command to generate PCB fabrication files.  

If you are using footprint and symbol library management commands you may also need to set `$env.kiops_lib_location`. The default is `../cuprous-kicad-libs` relative to the `kiops` directory.

Executables used by `kiops.nu` are picked up from the `./target/release` directory.  This can be changed by setting `$env.kiops_bin`.