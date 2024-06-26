#!/usr/bin/env python3
# Copyright lowRISC contributors (OpenTitan project).
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0
"""Collect the one-line summaries of all hardware blocks and output them as
Markdown-formatted table in the README.md at the top of the hardware block tree.
"""

from dataclasses import dataclass
import hjson
import logging as log
import os
import pathlib
from tabulate import tabulate
from typing import List
import unittest


def main():
    this_file_path = pathlib.Path(__file__)
    repo_root = this_file_path.parent.parent.resolve()

    # Collect entries for the HW summary from directories under hw/ip.
    hw_ip_dir = repo_root / 'hw' / 'ip'
    summary_entries = []
    for p in os.scandir(hw_ip_dir):
        if not p.is_dir():
            continue

        # Load data file, and skip blocks that don't provide one.
        name = p.name
        data_path = pathlib.Path(p) / 'data' / f'{name}.hjson'
        if not data_path.exists():
            log.warning(f'Skipping `{name}` because data file not found.')
            continue
        with open(data_path, 'r') as f:
            data = hjson.loads(f.read())

        name = data['name']

        # Get one-line description, and skip blocks that don't have one.
        if 'one_line_desc' not in data:
            log.warning(f'Skipping `{name}` because no `one_line_desc` found '
                        'in its data file.')
            continue
        one_liner = data['one_line_desc']

        # Create HW summary entry and append it to our list.
        summary_entries.append(HwSummaryEntry(name, one_liner))

    # Create table summarizing HW blocks.
    summary_entries.sort()
    table = HwSummaryTable(summary_entries)

    # Write table into hw/ip/README.md, replacing any content that was
    # previously in the section marked as autogenerated by this tool.
    with open(hw_ip_dir / 'README.md', 'r') as f:
        text = f.read()

        def replace_section_in_this_text(title: str, new_section: str):
            new_section = (
                '<!-- Do NOT edit this section manually, '
                f'it is generated by {os.path.relpath(this_file_path, repo_root)}. '
                '-->\n') + new_section
            nonlocal text
            text = replace_section_in_text(
                text,
                f'\n<!-- BEGIN AUTOGEN {title} -->\n',
                f'\n<!-- END AUTOGEN {title} -->\n',
                new_section)

        replace_section_in_this_text(
            'Summary Table',
            table.to_markdown(shortcut_ref_link=True).rstrip())
        replace_section_in_this_text(
            'README Link Reference Definitions',
            table.readme_link_references_markdown()
        )

    with open(hw_ip_dir / 'README.md', 'w') as f:
        f.write(text)


@dataclass
class HwSummaryEntry:
    """Simple dataclass summarizing one HW block."""
    block_name: str
    one_line_desc: str

    def __lt__(self, other):
        return self.block_name < other.block_name

    def path_to_readme(self):
        return f'./{self.block_name}/README.md'

    def name_linked_to_readme_markdown(self, shortcut_ref_link: bool) -> str:
        s = f'[`{self.block_name}`]'
        if not shortcut_ref_link:
            s += f'({self.path_to_readme()})'
        return s

    def link_reference_to_readme_markdown(self) -> str:
        return f'[`{self.block_name}`]: {self.path_to_readme()}'


@dataclass
class HwSummaryTable:
    """Table summarizing multiple hardware blocks, can be formatted as Markdown.
    """
    entries: List[HwSummaryEntry]

    def to_markdown(self, shortcut_ref_link: bool) -> str:
        header = ['HW Block', 'Brief Summary']
        data = [[e.name_linked_to_readme_markdown(shortcut_ref_link),
                 e.one_line_desc]
                for e in self.entries]
        return tabulate(data, header, tablefmt='github')

    def readme_link_references_markdown(self) -> str:
        data = [e.link_reference_to_readme_markdown() for e in self.entries]
        return '\n'.join(data)


def replace_section_in_text(text: str, start_marker: str, end_marker: str,
                            new_section: str) -> str:
    """Return a copy of text where everything between start_marker and
    end_marker is replaced with new_section.

    If start_marker does not exist, start_marker, new_section, and end_marker
    are appended at the end of text.  If end_marker does not exist, everything
    after start_marker is replaced with new_section followed by end_marker.
    """

    start_pos = text.find(start_marker)
    if start_pos < 0:
        # text does not contain start_marker, so append it and set start_pos to
        # the new end of text.
        text += start_marker
        start_pos = len(text)
    else:
        start_pos += len(start_marker)
    # start_pos is now set to the end of start_marker.

    end_pos = text.find(end_marker)
    if end_pos < 0:
        # text does not contain end_marker, so set end_pos to the current end of
        # text and append end_marker.
        end_pos = len(text)
        text += end_marker
    # end_pos is now set to the beginning of end_marker.

    assert start_pos <= end_pos, (
        "text seems to contain end_marker but not start_marker, "
        "which makes this function undefined")

    text = text[0:start_pos] + new_section + text[end_pos:]

    return text


class TestReplaceSectionInText(unittest.TestCase):
    def test_simple(self):
        self.assertEqual(
            replace_section_in_text('beforeSTARTfooENDafter', 'START', 'END',
                                    'bar'),
            'beforeSTARTbarENDafter'
        )

    def test_neither_end_nor_start(self):
        self.assertEqual(
            replace_section_in_text('before', 'START', 'END', 'bar'),
            'beforeSTARTbarEND'
        )

    def test_no_end(self):
        self.assertEqual(
            replace_section_in_text('beforeSTARTfoo', 'START', 'END', 'bar'),
            'beforeSTARTbarEND'
        )


if __name__ == '__main__':
    main()
