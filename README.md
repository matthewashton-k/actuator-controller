## Description

Controller for a raspberry pi pico running [this](https://github.com/utahrobotics/lunadev-2025/tree/main/embedded/actuator) actuator controller firmware.

I made this project as a way to test the actuator before I have finished the functionality of controlling the actuator from lunabase.

## Usage
1. Find where the pico is connected to. Likely it is /dev/ttyACM*
2. Then edit main.rs to replace this path with the actual path ```let port_path = "/dev/tnt1";```
3. Execute ```cargo run```


You should now be able to change the speed and direction of the actuator through the terminal interface.


## WARNING

I have not actually tested this, or the firmware on the actual pico yet.