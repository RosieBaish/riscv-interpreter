document.getElementById('memory-download').addEventListener('click', downloadFile);

function downloadAllMemory() {
    /** @type {Memory} */
    var memory = globalP.getMemory();
    var zero = 0;
    var byte = [0,0,0,0];
    var fileArr = [];
    var item = 0;
    for (var i = 0; i < Math.pow(2,20); i++) {
        var mem = memory.getMem(i);
        if (mem != 0) {
            byte[i % 4] = mem;
        }
        if (i % 4 == 3) {
            if (byte.every(function(ele){return ele == 0})) {
                zero += 1;
            }
            else {
                if (zero == 0) {
                    var string = sprintf("%02x%02x%02x%02x", byte[3],byte[2],byte[1],byte[0]);
                    string = string.replace(/^0+/, '');
                    fileArr.push(string);
                }
                else {
                    if (zero <= 3) {
                        for (var z = 0; z  < zero; z++) {
                            fileArr.push("0");
                        }
                    }
                    else {
                        fileArr.push(sprintf("%s*0", zero));
                    }
                    var string = sprintf("%02x%02x%02x%02x", byte[3],byte[2],byte[1],byte[0]);
                    string = string.replace(/^0+/, '');
                    fileArr.push(string);
                }
                zero = 0;
            }
            byte = [0,0,0,0];
        }
    }
    var fileString = "v2.0 raw\n";
    for (var i = 0; i < fileArr.length; i++) {
        // special condition endwith 0 0 0
        if (i == fileArr.length - 1 && fileArr[i].endsWith("*0")) {
            break;
        }
        fileString += fileArr[i];
        fileString += " ";
        if (i % 8 == 7) {
            fileString += "\n";
        }
    }
    if (fileString == "v2.0 raw\n") {
        fileString = "v2.0 raw\n0";
    }
    return fileString.trim();
}

function downloadFile() {
    var blob = new Blob([downloadAllMemory()], {type: "text/plain;charset=utf-8"});
    saveAs(blob, "memory.txt");
}