# unifi-search-tool
Tool to search for device by MAC Address across sites within a Unifi controller

## Change default controller URL

1. Edit the line in ```search-unifi-tool.py``` that is commented out. Currently, it looks like this: ```#server_input.setText('https://<your_unifi_domain_here>:8443')``` 
Un-comment the line, and put in the domain of your unifi controller. This will place the url into the ```Server URL``` field without needing to type it in each time the application is launched.

2. Re-compile the code using the instructions listed in [Build From Source](#build-from-source)

## Build From Source

Requirements: PyQt5, unifi_api, pyinstaller

1. Download the Zip of the source files and extract it

2. Open up a terminal in the directory

3. Run ```pyinstaller --onefile --windowed --icon=unifi-search.ico search-unifi-tool.py``` in the terminal

4. Go to the ```dist``` directory to find the .exe file

NOTE: If you omit the ```--onefile``` argument, it will provide the application and its subdirectories. The application will still function the same, everything will just be unpacked instead of in a single .exe file.
