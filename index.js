import { get_endpoints_js, test_city } from "./pkg";
import 'bootstrap/dist/css/bootstrap.min.css';

class ProgressBar {
    constructor(progressBars, name) {
        this.name = name;
        // Spinner
        this.pbSpinner = document.createElement("div");
        this.pbSpinner.id = "progress-bar-" + name + "-spinner";
        this.pbSpinner.innerHTML =
            '<div class="spinner-border spinner-border-sm" role="status"><span class="visually-hidden">Loading external list</span></div>';
        progressBars.appendChild(this.pbSpinner);
        // Name, e.g. paper
        this.pbType = document.createElement("div");
        this.pbType.id = "progress-bar-" + name + "-type";
        this.pbType.appendChild(document.createTextNode(name));
        progressBars.appendChild(this.pbType);
        // Counter, e.g. 3/10
        this.pbCounter = document.createElement("div");
        this.pbCounter.id = "progress-bar-" + name + "-counter";
        this.pbCounter.appendChild(document.createTextNode("0"));
        progressBars.appendChild(this.pbCounter);
        // Last message
        this.pbMessage = document.createElement("div");
        this.pbMessage.id = "progress-bar-" + name + "-message";
        this.pbMessage.appendChild(document.createTextNode(""));
        progressBars.appendChild(this.pbMessage);
    }
}

class UI {
    constructor() {
        this.progressBarsDiv = document.getElementById("progress-bars");
        this.channel = new MessageChannel();
        this.channel.port1.onmessage = this.update.bind(this);
        this.progressBars = {};
    }

    update(message) {
        message = message.data;
        try {
            if (message["message_type"] === "progress-bar-add") {
                this.progressBars[message["name"]] = new ProgressBar(
                    this.progressBarsDiv,
                    message["name"]
                );
            } else if (message["message_type"] === "progress-bar-update") {
                this.progressBars[message["name"]].pbCounter.textContent =
                    message["counter"];
            } else if (message["message_type"] === "progress-bar-message") {
                this.progressBars[message["name"]].pbMessage.textContent =
                    message["message"];
            } else if (message["message_type"] === "progress-bar-finish") {
                this.progressBars[message["name"]].pbMessage.textContent =
                    message["message"];
                // Stop spinner
                this.progressBars[message["name"]].pbSpinner.textContent = "✅";
                delete this.progressBars[message["name"]];
            } else if (message["message_type"] === "problem-report") {
                let problem_lis = message["problems"].map((problem) => {
                    let li = document.createElement("li");
                    // textContent to avoid XSS
                    li.textContent = problem;
                    return li;
                });
                let ul = document.getElementById("problem-report");
                // Swap old and new report
                ul.replaceChildren(...problem_lis);
                let header = document.getElementById("problem-report-header");
                if (message["is_final"]) {
                    header.innerText = "Validierungsreport";
                } else {
                    header.innerText = "Zwischenstand Validierungsreport";
                }
            } else {
                alert("Invalid message: " + message);
            }
        } catch (e) {
            console.log("Failed to handle message: ", message);
            console.log("Error: " + e);
        }
    }
}

/** Load the list of known endpoints from github and show them in the dropdown */
async function init_endpoint_list() {
    console.log("Loading endpoints from github");
    let endpoints = await get_endpoints_js();
    // <option disabled selected>... oder Stadt auswählen</option>
    let placeholder_option = document.createElement("option");
    placeholder_option.setAttribute("disabled", true);
    placeholder_option.setAttribute("selected", true);
    placeholder_option.innerText = "... oder Stadt auswählen";
    let options = endpoints.map((endpoint) => {
        let option = document.createElement("option");
        // 0: name, 1: url
        option.value = endpoint[1];
        option.innerText = endpoint[0] + " (" + endpoint[1] + ")";
        return option;
    });
    options.unshift(placeholder_option);
    document.getElementById("city-select").replaceChildren(...options);
    // Dropdown is disabled initially, we activate it once we loaded the data
    document.getElementById("city-select").removeAttribute("disabled");
}

function add_handler() {
    console.log("Init js handler");
    document.getElementById("city-select").addEventListener("change", (ev) => {
        document.getElementById("city-url").value = ev.target.value;
    });
    console.log("Init wasm ui");
    let ui = new UI();
    document.getElementById("run").addEventListener("click", async (ev) => {
        ev.preventDefault();
        let url = document.getElementById("city-url").value;
        console.log("Running city " + url);
        await test_city(url, ui.channel);
    });
    // noinspection JSIgnoredPromiseFromCall
    init_endpoint_list();
}

(() => {
    // Not sure what's going with this otherwise
    if (!window.wasm_did_init) {
        window.wasm_did_init = true;
        add_handler();
    }
})();
