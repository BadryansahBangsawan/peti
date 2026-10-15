const menu = document.querySelector(".menu");
const links = document.querySelector(".nav-links");

if (menu && links) {
  menu.addEventListener("click", () => {
    const open = links.classList.toggle("is-open");
    menu.setAttribute("aria-expanded", open ? "true" : "false");
    menu.setAttribute("aria-label", open ? "Tutup menu" : "Buka menu");
  });
}
