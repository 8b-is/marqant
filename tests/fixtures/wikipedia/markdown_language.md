# Markdown

**Markdown** is a lightweight markup language that you can use to add formatting elements to
plaintext text documents. Created by John Gruber in 2004, Markdown is now one of the world's
most popular markup languages.

## History

### Creation

John Gruber created the Markdown language in 2004 in collaboration with Aaron Swartz, with the
goal of enabling people to write using an easy-to-read and easy-to-write plain text format that
could be converted to structurally valid HTML.

Gruber's original Markdown specification was implemented in Perl. The syntax was influenced by
existing conventions for marking up plain text in email, and it was designed to be as readable
as possible without sacrificing expressiveness.

### Evolution

Since its creation, Markdown has evolved into many dialects and implementations:

1. **CommonMark**: A standardized specification addressing ambiguities in the original
2. **GitHub Flavored Markdown (GFM)**: Extends CommonMark with tables, strikethrough, and task lists
3. **MultiMarkdown**: Adds footnotes, tables, and citation support
4. **Pandoc Markdown**: A superset with many extensions for academic writing
5. **R Markdown**: Extends Markdown for data science documents

## Basic Syntax

### Headings

Headings are created by adding hash symbols (`#`) before a word or phrase:

```markdown
# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6
```

### Emphasis

You can add emphasis with asterisks or underscores:

- **Bold**: `**text**` or `__text__`
- *Italic*: `*text*` or `_text_`
- ***Bold and italic***: `***text***`
- ~~Strikethrough~~: `~~text~~`

### Lists

#### Unordered Lists

Unordered lists use hyphens, asterisks, or plus signs:

- Item 1
- Item 2
  - Nested item 2a
  - Nested item 2b
- Item 3

#### Ordered Lists

Ordered lists use numbers followed by periods:

1. First item
2. Second item
3. Third item
   1. Nested first item
   2. Nested second item
4. Fourth item

### Links

Markdown supports two styles of links:

- Inline: `[Link text](URL "Optional title")`
- Reference: `[Link text][reference]` with `[reference]: URL` elsewhere

### Images

Images use a similar syntax to links with a preceding exclamation mark:

```markdown
![Alt text](image.png "Optional title")
```

### Code

Inline code is wrapped in backticks: `` `code` ``

Code blocks can be created with triple backticks and optional language identifier:

```python
def hello_world():
    print("Hello, World!")
    return True
```

### Blockquotes

Blockquotes are created with the `>` character:

> This is a blockquote.
>
> It can span multiple paragraphs.
>
> > Nested blockquotes are also supported.

### Tables (GFM Extension)

Tables are supported in GitHub Flavored Markdown:

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |

Alignment can be controlled with colons:

| Left | Center | Right |
|:-----|:------:|------:|
| A    |   B    |     C |

### Task Lists (GFM Extension)

Task lists create interactive checkboxes in some renderers:

- [x] Completed task
- [ ] Incomplete task
- [x] Another completed task

## Extended Syntax

### Footnotes

Some Markdown processors support footnotes:

```markdown
Here is a sentence with a footnote.[^1]

[^1]: This is the footnote content.
```

### Definition Lists

Definition lists group terms with their definitions:

```markdown
Term
: Definition
```

### Emoji

Many Markdown processors support emoji shortcodes:

```markdown
:smile: :heart: :thumbsup:
```

## Applications

Markdown is used extensively in:

- **Documentation**: README files, wikis, and technical documentation
- **Blogging**: Static site generators like Jekyll, Hugo, and Gatsby
- **Note-taking**: Apps like Obsidian, Notion, and Bear
- **Academic writing**: With tools like Pandoc and Overleaf
- **Chat platforms**: Discord, Slack, and many messaging applications
- **Issue trackers**: GitHub, GitLab, and Jira
- **AI systems**: Efficient context representation with tools like Marqant

## Implementations

There are hundreds of Markdown implementations in virtually every programming language:

| Language | Library |
|----------|---------|
| Python | Python-Markdown, mistune |
| JavaScript | marked, markdown-it, showdown |
| Ruby | kramdown, Redcarpet |
| Java | flexmark-java, CommonMark |
| Go | goldmark, blackfriday |
| Rust | pulldown-cmark, comrak |
| PHP | Parsedown, PHP Markdown |

## Standardization Efforts

The lack of a formal specification led to significant variation across implementations.
CommonMark, developed by John MacFarlane and a team of contributors, aims to address this
with:

1. An unambiguous specification
2. A comprehensive test suite with 652 tests
3. Reference implementations in multiple languages
4. Active community maintenance

## See Also

- reStructuredText
- AsciiDoc
- LaTeX
- HTML

## References

1. Gruber, J. (2004). Markdown. *Daring Fireball*. Retrieved from daring fireball.
2. MacFarlane, J. et al. (2021). CommonMark Spec Version 0.30.
3. GitHub. (2017). *GitHub Flavored Markdown Spec*. GitHub, Inc.
