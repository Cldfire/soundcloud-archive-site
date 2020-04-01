
async function submit_NewUser(event){
    
    const response = await fetch(
        "/api/register",
        {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            credentials: 'same-origin',
            body: JSON.stringify({
                password: event.target.UsernameNewUser.value,
                username: event.target.Password1.value,
            })
        }
    );    
    if (response.ok) {
        const userInfo = await response.json();
        signedIn.set(true);
        userId.set(userInfo.user_id);
        navigateTo('/')
    } else {
        // TODO: handle potential errors / issues
        // should reply with json payload
        alert("Send Fail");
    }
    window.location.replace("SC_Oauth.html");

}

async function submit_SC_OAuth(){

}