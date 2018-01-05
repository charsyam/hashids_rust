var Hashids = require('hashids');

args = process.argv.slice(1);
var salt = '';
var minLength = 0;
var alphabet = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890';
var numbers = [];

for (var i = 0; i < args.length; i++) {
    var element = args[i];
    
    if (element === "-s" || element === "--salt") {
        salt = args[++i]
    } else if (element === "-m" || element === "--min-length") {
        minLength = parseInt(args[++i])
    } else if (element === "-a" || element === "--alphabet") {
        alphabet = args[++i]
    } else {
        numbers.push(parseInt(element));
    }
}

var hashids = new Hashids(salt, minLength, alphabet);
var encoded = hashids.encode.call(hashids, numbers);
console.log(encoded);