console.log("Hello, world!");
// a is defined earlier in a runtime call to js::exec
console.log(a + 2);
let el = document.getElementById('getme');
console.log(el);
let ps = document.querySelectorAll('p');
console.log(ps);
console.log(el.getAttribute('id'));
el.setAttribute('id', 'got');
console.log(el.getAttribute('id'));
let bad = new Node(18);
console.log(bad.getAttribute('uh-oh'));
console.log(document.getElementById('dne'));
