export function lazyload(image, src) {
    const loaded = () => {
        image.classList.add("visible");
    }

    const observer = new IntersectionObserver(entries => {
        if (entries[0].isIntersecting) {
            console.log("lazy loaded an image");
            image.src = src;
            if (image.complete) {
                loaded();
            } else {
                image.addEventListener("load", loaded);
            }
        }
    }, {root: null, rootMargin: "0px", threshold: 0});
    observer.observe(image);
    return {
        destroy() {
            image.removeEventListener("load", loaded);
        }
    }   
}