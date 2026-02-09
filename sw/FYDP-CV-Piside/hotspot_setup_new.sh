#!/bin/bash

# Clean up previous attempts
sudo nmcli con delete Hotspot || true
sudo systemctl stop dnsmasq 
sudo systemctl disable dnsmasq 

# 1. Create the connection profile
sudo nmcli con add type wifi ifname wlan0 con-name Hotspot autoconnect yes ssid rpi_whynowork2 

# 2. Set mode and band
sudo nmcli con modify Hotspot 802-11-wireless.mode ap 802-11-wireless.band bg 

# 3. Configure IP settings
# Using 'shared' is correct, but we'll specify a slightly different range if needed
sudo nmcli con modify Hotspot ipv4.method shared ipv4.addresses 192.168.2.1/24

# 4. Security: WPA2-AES (CCMP) is the gold standard for Win 11 stability
sudo nmcli con modify Hotspot wifi-sec.key-mgmt wpa-psk 
sudo nmcli con modify Hotspot wifi-sec.psk everydaysameworse 

# 5. Fix Windows 11 Compatibility (Crucial Changes)
# PMF = 1 (Optional) is usually safer than 2 (Required) for broad compatibility
sudo nmcli con modify Hotspot 802-11-wireless-security.proto rsn 
sudo nmcli con modify Hotspot 802-11-wireless-security.group ccmp 
sudo nmcli con modify Hotspot 802-11-wireless-security.pairwise ccmp 
sudo nmcli con modify Hotspot 802-11-wireless-security.pmf 1

# 6. Disable IPv6 (Windows 11 often drops connections if IPv6 is 'searching' but failing)
sudo nmcli con modify Hotspot ipv6.method ignore

# 7. Bring it up
sudo nmcli con up Hotspot 

# 8. Persistent Power Management Fix
# NetworkManager can override 'iw', so we set it via nmcli as well
sudo nmcli connection modify Hotspot 802-11-wireless.powersave 2
sudo iw dev wlan0 set power_save off

# 9. Cleanup Hostapd interference
sudo systemctl stop hostapd 
sudo systemctl mask hostapd