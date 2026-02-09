# Unmask and enable WiFi client services
sudo systemctl unmask wpa_supplicant
sudo systemctl enable wpa_supplicant
sudo systemctl start wpa_supplicant

# Re-enable networking services
sudo systemctl enable NetworkManager  # If you have NetworkManager
# Stop and disable hostapd
sudo systemctl stop hostapd
sudo systemctl disable hostapd
# Bring wlan0 back up with DHCP
sudo ip link set wlan0 down
sleep 2
sudo ip link set wlan0 up