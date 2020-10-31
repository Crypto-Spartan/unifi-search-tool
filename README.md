# Unifi Search Tool v1.4.1 - Download [Here](https://github.com/Crypto-Spartan/unifi-search-tool/releases/latest)
Does your unifi controller have lots of sites? Do you frequently have equipment returned from those sites and you can't remember where it's adopted in the controller? Enter Unifi Search Tool.

### How to Use

![example](https://raw.githubusercontent.com/Crypto-Spartan/unifi-search-tool/master/screenshots/example.png "example")

1. Enter your username & password for your Unifi Controller

2. Enter your Unifi Controller domain/IP. You must include the proper http:// or https:// with the appropriate port number at the end, unless it runs on 80/443. (You will see this in the address bar of your browser when you open up your Unifi Controller.)

3. Enter the MAC Address of the device you're searching for

4. Click search

5. Profit

The tool will tell you which site in the controller that the device was adopted to. If it hasn't been adopted, the tool will tell you that the device could not be found.

## **Advanced**

### NEW - Pre-populate username, password, & URL fields

These instructions are for those that would like to add in their own defaults so that they don't need to re-enter their credentials or controller URL each time the program is opened. (This will only work for the installed version unless you decide to build the portable version from source.)

#### NOTE: If you choose to do this and credentials are stolen, I am not responsible. This is at your own risk.

1. Find `config.txt` within the install folder. (Default is C:\Program Files (x86)\Unifi Search Tool)

2. Add in your own values to the right of the `=` symbol

3. Save config.txt

Next time Unifi Search Tool is launched, it will have new pre-populated fields.

### Build From Source

Requirements: PyQt5, pyinstaller, [unifi-python-api](https://github.com/r4mmer/unifi_python_api)

1. Download the Zip of the source files and extract it

2. Open up a terminal in the directory

3. Run `pyinstaller --onefile --windowed --icon=unifi-search.ico search-unifi-tool.py` in the terminal

4. Go to the `dist` directory to find the .exe file

NOTE: If you omit the `--onefile` argument, it will provide the application and its subdirectories. The application will still function the same, everything will just be unpacked instead of in a single .exe file.
