import resources, sys
from concurrent.futures import ThreadPoolExecutor

from PyQt5.QtWidgets import QApplication, QLabel, QWidget, QLineEdit, QGridLayout, QPushButton, QMessageBox
from PyQt5 import QtGui

from unifi_api import UnifiClient
from unifi_api.utils import models
from unifi_api.utils.decorators import requires_login, guard
from unifi_api.utils.exceptions import UnifiLoginError
import trafaret

login_success = False

class unifi_api_tosearch(UnifiClient):
    @guard(device_mac=models.MacAddress)
    def find_device_by_mac(self, device_mac):
        for s in self.list_sites():
            if device_mac in [models.MacAddress(d['mac']) for d in self.list_devices(site=s['name'])]:
                return s['desc']

  
def background_thread(client, mac_input):
    try:
        site_name = client.find_device_by_mac(mac_input.text())
    except trafaret.base.GuardError:
        site_name = 'MAC Address Error'
        fail = True
    else:
        fail = False
    
    return site_name, fail


def on_search_click():
    global login_success
    client = unifi_api_tosearch(server_input.text())
    client.login(username=user_input.text(), password=pass_input.text())
    msgBox = QMessageBox()

    if not login_success:
        try:
            client.site_stat_5min()
        except:
            msgBox.warning(window, "Login Error", "Incorrect username/password")
        else:
            login_success = True

    if login_success:
        with ThreadPoolExecutor() as executor:
            future = executor.submit(background_thread, client, mac_input)
            site_name, fail = future.result()
        
        if fail:
            msgBox.about(window, "Input Error", "Invalid MAC Address")
            login_success = False
        elif site_name is None:
            msgBox.about(window, "Not Found", "Device not found in controller.")
        else:
            msgBox.about(window, "Found", f'The device with MAC {mac_input.text()} belongs to the "{site_name}" site.')

    else:
        msgBox.warning(window, "Login Error", "Unable to login")
        login_success = False


if __name__ == '__main__':

    prepopulated = {}
    key_check = {'user','pass','url'}
    try:
        with open('config.txt', 'r') as f:
            for line in f.readlines():
                line = line.strip()
                try:
                    line = line.split('=')
                    key = line[0].strip()
                    value = line[1].strip()
                except:
                    pass
                else:
                    if key in key_check:
                        prepopulated[key] = value
    except FileNotFoundError:
        pass

    app = QApplication(sys.argv)

    window = QWidget()
    window.setWindowTitle('Unifi Search Tool - v1.4.1')
    window.resize(400,200)

    layout = QGridLayout()

    label1 = QLabel('Enter Unifi Controller Credentials')
    layout.addWidget(label1, 0, 0, 1, 2)

    user_label = QLabel('Username')
    layout.addWidget(user_label, 1, 0)
    user_input = QLineEdit()
    layout.addWidget(user_input, 1, 1)

    pass_label = QLabel('Password')
    layout.addWidget(pass_label, 2, 0)
    pass_input = QLineEdit()
    pass_input.setEchoMode(2)
    layout.addWidget(pass_input, 2, 1)

    server_label = QLabel('Server URL')
    layout.addWidget(server_label, 3, 0)
    server_input = QLineEdit()
    layout.addWidget(server_input, 3, 1)

    mac_label = QLabel('MAC Address')
    layout.addWidget(mac_label, 4, 0)
    mac_input = QLineEdit()
    layout.addWidget(mac_input, 4, 1)

    button_search = QPushButton('Search Unifi')
    layout.addWidget(button_search, 5, 0, 1, 2)

    input_fields = {
        'user': user_input,
        'pass': pass_input,
        'url': server_input
    }

    for key in prepopulated:
        field = input_fields.get(key, None)
        field.setText(prepopulated[key])
    

    button_search.clicked.connect(on_search_click)

    app.setWindowIcon(QtGui.QIcon(':/icons/unifi-search.ico'))
    window.setLayout(layout)              
    window.show()
    sys.exit(app.exec_())
