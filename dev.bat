cargo build --target i686-pc-windows-msvc
del "C:\Program Files (x86)\Galaxy On Fire 2\d3d9.dll"
copy target\i686-pc-windows-msvc\debug\d3d9.dll "C:\Program Files (x86)\Galaxy On Fire 2"
