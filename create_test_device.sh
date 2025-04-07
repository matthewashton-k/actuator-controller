#!/usr/bin/env bash

# Load necessary modules
sudo modprobe libcomposite
sudo modprobe usb_f_acm

# Create a USB gadget
cd /sys/kernel/config/usb_gadget/
sudo mkdir -p myacm
cd myacm

# Configure the USB device
sudo bash -c 'echo "0x1d6b" > idVendor'  # Linux Foundation
sudo bash -c 'echo "0x0104" > idProduct'  # ACM gadget
sudo mkdir -p strings/0x409
sudo bash -c 'echo "fedcba9876543210" > strings/0x409/serialnumber'
sudo bash -c 'echo "Your Name" > strings/0x409/manufacturer'
sudo bash -c 'echo "Test ACM Device" > strings/0x409/product'

# Create configuration
sudo mkdir -p configs/c.1/strings/0x409
sudo bash -c 'echo "ACM Config" > configs/c.1/strings/0x409/configuration'

# Create the ACM function
sudo mkdir -p functions/acm.usb0
sudo ln -s functions/acm.usb0 configs/c.1/

# Enable the gadget by binding it to a UDC (USB Device Controller)
# You need to use the name of your system's UDC, which varies by hardware
UDC=$(ls /sys/class/udc | head -n 1)
sudo bash -c "echo $UDC > UDC"