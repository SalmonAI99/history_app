const pages = [
    {
        title: "Chapter 1",
        content: `
            <p>It was a beautiful morning.</p>

            <p>
                The town was quiet, and the streets were
                still empty.
            </p>

            <p>
                Emma opened the window and looked outside.
            </p>
        `
    },

    {
        title: "Chapter 2",
        content: `
            <p>Emma walked down the old road.</p>

            <p>
                She had never been this far from home before.
            </p>

            <p>
                At the end of the road stood an enormous library.
            </p>
        `
    },

    {
        title: "Chapter 3",
        content: `
            <p>The library doors slowly opened.</p>

            <p>
                Inside, thousands of books filled the shelves.
            </p>

            <p>
                Emma smiled.
            </p>
        `
    }
];

let currentPage = 0;

const content = document.getElementById("book-content");
const title = document.getElementById("chapter-title");
const pageNumber = document.getElementById("page-number");

const previousButton = document.getElementById("previous");
const nextButton = document.getElementById("next");


function showPage() {
    const page = pages[currentPage];

    title.textContent = page.title;
    content.innerHTML = page.content;

    pageNumber.textContent =
        `${currentPage + 1} / ${pages.length}`;

    previousButton.disabled = currentPage === 0;
    nextButton.disabled = currentPage === pages.length - 1;
}


nextButton.addEventListener("click", () => {
    if (currentPage < pages.length - 1) {
        currentPage++;
        showPage();
    }
});


previousButton.addEventListener("click", () => {
    if (currentPage > 0) {
        currentPage--;
        showPage();
    }
});
async function loadBook() {
    const response = await fetch(
        "/api/book"
    );

    const book = await response.text();
    content.textContent = book;

}

loadBook();
