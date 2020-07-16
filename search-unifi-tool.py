import sys
import resources

from PyQt5.QtWidgets import QApplication, QLabel, QWidget, QLineEdit, QGridLayout, QPushButton, QMessageBox
from PyQt5 import QtGui

from unifi_api import UnifiClient
from unifi_api.utils import models
from unifi_api.utils.decorators import requires_login, guard
from unifi_api.utils.exceptions import UnifiLoginError

class unifi_api_tosearch(UnifiClient):
    @guard(device_mac=models.MacAddress)
    def find_device_by_mac(self, device_mac):
        for s in self.list_sites():
            if device_mac in [models.MacAddress(d['mac']) for d in self.list_devices(site=s['name'])]:
                return s['desc']

app = QApplication(sys.argv)

window = QWidget()
window.setWindowTitle('Unifi Search Tool')
window.resize(400,200)

layout = QGridLayout()

label1 = QLabel('Enter Unifi Controller Credentials')
layout.addWidget(label1, 0, 0, 1, 2)

user_label = QLabel('Username')
layout.addWidget(user_label, 1, 0)
user_input = QLineEdit()
#user_input.setText('<your_username_here>')
layout.addWidget(user_input, 1, 1)

pass_label = QLabel('Password')
layout.addWidget(pass_label, 2, 0)
pass_input = QLineEdit()
pass_input.setEchoMode(2)
#pass_input.setText('<your_password_here>')
layout.addWidget(pass_input, 2, 1)

server_label = QLabel('Server URL')
layout.addWidget(server_label, 3, 0)
server_input = QLineEdit()
#server_input.setText('https://<your_unifi_domain_here>:8443')
layout.addWidget(server_input, 3, 1)

mac_label = QLabel('MAC Address')
layout.addWidget(mac_label, 4, 0)
mac_input = QLineEdit()
layout.addWidget(mac_input, 4, 1)

button_search = QPushButton('Search Unifi')
layout.addWidget(button_search, 5, 0, 1, 2)

def on_search_click():

    login_success = False
    client = unifi_api_tosearch(server_input.text())
    client.login(username=user_input.text(), password=pass_input.text())
    msgBox = QMessageBox()

    try:
        client.site_stat_5min()
    except:
        QMessageBox.warning(window, "Error", "Incorrect username/password")
    else:
        login_success = True

    if login_success:
        site_name = client.find_device_by_mac(mac_input.text())
        
        if site_name is None:
            QMessageBox.about(window, "Not Found", "Device not found in controller.")
        else:
            QMessageBox.about(window, "Found", f'The device with MAC {mac_input.text()} belongs to the "{site_name}" site.')

    else:
        pass
    
button_search.clicked.connect(on_search_click)

app.setWindowIcon(QtGui.QIcon(':/icons/unifi-search.ico'))
window.setLayout(layout)              
window.show()
sys.exit(app.exec_())
