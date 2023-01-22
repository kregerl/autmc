export function removeClassname(id: string, classname: string) {
    let element = document.getElementById(id);
    element.classList.remove(classname);
}

export function updateSelectionClasses(id: string, buttons: string[]): string{
    let element = document.getElementById(id);
    for (let i = 0; i < buttons.length; i++) {
        let button = buttons[i];
        if (button != id) {
            removeClassname(button, "selected");
        }
    }
    element.classList.add("selected");
    console.log("element", element);
    return id;
}

export const SELECTION_CLASSNAME = "selected";
