This crate provides an API to be used with the modified Lora simulator LWN-Simulator 
[see LWN-Simulator](https://github.com/UniCT-ARSLab/LWN-Simulator)
[see LWN-Simulator extension](https://github.com/windy40/LWN-Simulator/tree/mcu-api-siov4)

This API allows a rust process to link to a device component in the simulator and use it as a LORA module to send and receive Lora messages to/from a Lora network.

The API comm functions mimick the [PYCOM Lora network interface](https://docs.pycom.io/firmwareapi/pycom/network/lora/) (PYCOM dev boards are micropython programmable devices).

The intent is to simulate a LORA-capable device as a rust CLI program.

This is still work in progress, but I would be happy to share with anyone interested in the simulation of Lora devices.