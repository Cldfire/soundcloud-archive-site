const ID = {
    User: document.getElementByID("Username"),
    Pass: document.getElementByID("Password"),
};


async function submit_NewUser(){
    let user = document.getElementById("Username").textContent;
    let Password1 = document.getElementById("Password1").textContent;
    let Password2 = document.getElementById("Password2").textContent;

    window.location.href = "SC_Oauth.html"

}

