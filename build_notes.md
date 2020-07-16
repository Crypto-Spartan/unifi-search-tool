## To build installer:

1. Open terminal in directory
2. Run `pyinstaller --windowed --icon=unifi-search.ico search-unifi-tool.py`
3. Change AppVersion in `build_installer.iss`
4. Use [Inno Setup Compiler](https://github.com/jrsoftware/issrc) with `build_installer.iss`
5. Result will be in `<dir>/Output/mysetup.exe`

## To build portable:

1. Open teminal in directory
2. Run `pyinstaller --windowed --onefile --icon=unifi-search.ico search-unifi-tool.py`
3. Result will be in `<dir>/build/unifi-search-tool.exe`