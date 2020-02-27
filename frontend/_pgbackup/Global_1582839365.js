var Username;
var Password;

document.title = username();

function username(){
    if(Username == undefined)
        Username = "Bobby";
    document.title = Username;
}