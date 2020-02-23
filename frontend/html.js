const html_type = {
    music: 'music',
    block: 'b',
    paragraph: 'p',
    login: 'login',
    div: 'div',
    title: 'title',
};

class output{
    #html_type;
    #css_type;
    #data;
    #action;

    constructor(type, data, css= '', action = '') {
        this.html_type = type;
        this.css_type = css
        this.data = data;
        this.action = action;
    }

    get output(){

    }
}