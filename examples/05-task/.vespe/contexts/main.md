

propose a task that a system manager could need to do in 3 steps to configure a new server with iptables.

<!-- answer-30af8aa2-5594-4dfd-912e-912e4b8f2deb:begin { provider: 'gemini -y -m gemini-2.5-flash' }  -->
Here's a three-step task for a system manager to configure a new server with `iptables`:

**Task: Configure Basic `iptables` Firewall Rules**

**Step 1: Install `iptables-persistent` and Flush Existing Rules**
*   **Action:** Install the `iptables-persistent` package to ensure rules survive reboots, then flush all existing `iptables` rules to start with a clean slate.
*   **Command Example:**
    ```bash
    sudo apt update && sudo apt install -y iptables-persistent
    sudo iptables -F
    sudo iptables -X
    sudo iptables -Z
    ```

**Step 2: Define Essential Firewall Rules**
*   **Action:** Establish default policies (e.g., drop all incoming traffic), allow established connections, and open specific ports like SSH (port 22) and HTTP/HTTPS (ports 80, 443).
*   **Command Example:**
    ```bash
    sudo iptables -P INPUT DROP
    sudo iptables -P FORWARD DROP
    sudo iptables -P OUTPUT ACCEPT
    sudo iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
    sudo iptables -A INPUT -i lo -j ACCEPT
    sudo iptables -A INPUT -p tcp --dport 22 -j ACCEPT
    sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
    sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT
    ```

**Step 3: Save `iptables` Rules Permanently**
*   **Action:** Save the newly configured `iptables` rules so they are automatically loaded upon system reboot.
*   **Command Example:**
    ```bash
    sudo netfilter-persistent save
    sudo netfilter-persistent reload
    ```
<!-- answer-30af8aa2-5594-4dfd-912e-912e4b8f2deb:end {}  -->


