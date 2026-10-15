#!/usr/bin/env python3
"""Check rendered local files and HTML anchors after the strict MkDocs build."""

import argparse
from html.parser import HTMLParser
from pathlib import Path
import sys
from urllib.parse import unquote, urljoin, urlsplit

from mkdocs.config import load_config


ROOT = Path(__file__).resolve().parents[1]


class PageLinks(HTMLParser):
    def __init__(self, text):
        super().__init__(convert_charrefs=True)
        self.ids = set()
        self.links = []
        self.feed(text)

    def handle_starttag(self, tag, attributes):
        attributes = dict(attributes)
        if 'id' in attributes:
            self.ids.add(attributes['id'])
        if tag == 'a' and 'href' in attributes:
            self.links.append(attributes['href'])


def check_links(site_dir, site_url):
    site_dir = site_dir.resolve()
    pages = sorted(site_dir.rglob('*.html'))
    if not pages:
        return ['No rendered HTML pages found; run the strict MkDocs build first.'], 0, 0
    base_url = (site_url or 'https://documentation.invalid/').rstrip('/') + '/'
    site = urlsplit(base_url)
    prefix = site.path
    cache = {}
    errors = []
    checked = 0

    def parsed(path):
        if path not in cache:
            cache[path] = PageLinks(path.read_text(encoding='utf-8'))
        return cache[path]

    for page in pages:
        relative_page = page.relative_to(site_dir).as_posix()
        page_url = urljoin(base_url, relative_page)
        # Directory URLs make same-page anchors and sibling links resolve as hosted.
        if relative_page.endswith('index.html'):
            page_url = page_url.removesuffix('index.html')
        try:
            links = parsed(page).links
        except (OSError, UnicodeError) as exc:
            errors.append(f'{relative_page}: cannot read HTML: {exc}')
            continue
        for href in links:
            declared = urlsplit(href)
            resolved = urlsplit(urljoin(page_url, href))
            if resolved.scheme not in {'http', 'https'} or resolved.netloc != site.netloc:
                continue
            if not resolved.path.startswith(prefix):
                if not declared.scheme and not declared.netloc and not declared.path.startswith('/'):
                    errors.append(f'{relative_page}: relative link escapes the site prefix: {href}')
                continue
            relative = unquote(resolved.path[len(prefix):])
            target = (site_dir / relative).resolve()
            if not target.is_relative_to(site_dir):
                errors.append(f'{relative_page}: link escapes the rendered site: {href}')
                continue
            if resolved.path.endswith('/'):
                target /= 'index.html'
            checked += 1
            if not target.is_file():
                errors.append(f'{relative_page}: missing local file: {href}')
            elif resolved.fragment and target.suffix.lower() == '.html':
                try:
                    if unquote(resolved.fragment) not in parsed(target).ids:
                        errors.append(f'{relative_page}: missing local anchor: {href}')
                except (OSError, UnicodeError) as exc:
                    errors.append(f'{relative_page}: cannot read link target {href}: {exc}')
    return errors, checked, len(pages)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    args = parser.parse_args()
    try:
        config = load_config(config_file=str(args.root.resolve() / 'mkdocs.yml'))
        errors, count, pages = check_links(Path(config['site_dir']), config['site_url'])
    except (OSError, ValueError) as exc:
        print(f'Unable to check rendered documentation: {exc}', file=sys.stderr)
        return 1
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        return 1
    print(f'Checked {count} local links and anchors across {pages} rendered HTML pages.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
