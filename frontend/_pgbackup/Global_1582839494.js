var Username;
var Password;

document.title = username();

function username(){
    if(Username === null)
        Username = "Bobby";
    document.title = Username;
}