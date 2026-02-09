sudo systemctl stop dnsmasq
sudo systemctl disable dnsmasq
sudo systemctl restart NetworkManager

# 1. Create the connection profile
sudo nmcli con add type wifi ifname wlan0 con-name Hotspot autoconnect yes ssid rpi_whynowork2

# 2. Set the mode to Access Point (ap) and band to 2.4GHz (bg)
sudo nmcli con modify Hotspot 802-11-wireless.mode ap 802-11-wireless.band bg

# 3. Configure IP settings (Manual IP for the Pi, shared to others)
# We use 'shared' method so NM handles the DHCP for connecting devices
sudo nmcli con modify Hotspot ipv4.method shared ipv4.addresses 192.168.2.159/24

# 4. Set security to WPA2 (RSN) with a static key
sudo nmcli con modify Hotspot wifi-sec.key-mgmt wpa-psk
sudo nmcli con modify Hotspot wifi-sec.psk everydaysameworse

# 5. Fix Windows 11 Compatibility (Protocols and PMF)
sudo nmcli con modify Hotspot 802-11-wireless-security.proto rsn
sudo nmcli con modify Hotspot 802-11-wireless-security.group ccmp
sudo nmcli con modify Hotspot 802-11-wireless-security.pairwise ccmp
sudo nmcli con modify Hotspot 802-11-wireless-security.pmf 1

# 6. Bring it up
sudo nmcli con up Hotspot

sudo systemctl stop hostapd
sudo systemctl mask hostapd
sudo systemctl restart NetworkManager

sudo iw dev wlan0 set power_save off