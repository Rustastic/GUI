# GUI

## Overview
The **Graphical User Interface (GUI)** is a key component of the drone-based network simulation. It provides an intuitive and interactive way for users to observe and interact with the simulation in real time.

## Role in the Simulation
The GUI acts as the visual and control layer of the system. It bridges the gap between users and the underlying simulation logic, offering a clear view of network activity and allowing users to issue commands and monitor system behavior.

### Responsibilities
* **Visualization**: Displays the network structure and dynamic behavior of nodes, including:
  * Packet forwarding between nodes
  * Network discovery processes
* **User Interaction**: Allows users to interact with the simulation in an intuitive way by:
  * Sending commands to drones or other nodes
  * Adjusting simulation parameters in real time
* **System Feedback**: Reflects changes and responses from the network in real time, helping users understand the effects of their interactions.

### Features
* Real-time visualization of the drone-based network.
* Intuitive controls for managing nodes and issuing commands.
* Display of key simulation metrics and statistics.
* Seamless integration with the Simulation Controller and network components.
* Built using `egui`, `eframe` and `petgraph` for an interactive experience.
