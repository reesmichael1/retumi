console.log("Hello, world!");
let el = document.getElementById('getme');
console.log(el);
let ps = document.querySelectorAll('p');
console.log(ps);
console.log(el.getAttribute('id'));
el.setAttribute('id', 'got');
console.log(el.getAttribute('id'));
console.log(document.getElementById('dne'));
