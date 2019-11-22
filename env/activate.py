""" copy over env values from one file to main .env file """
import argparse
import json
from typing import Dict

from dotenv import dotenv_values


def format_dict(the_dict: Dict[str, str]):
    """ format dictionaries for rocket env variable
        - no quotes on keys
        - splits keys and values with "=" instead of ":"
    """
    inner = ', '.join(f'{key}={value}' for key, value in the_dict.items())
    start, end = '{', '}'
    return f'{start}{inner}{end}'


def db_string(db_to_url: Dict[str, str]) -> str:
    """ get single DB string from db --> url mapping """
    return format_dict({
        name: format_dict({'url': url})
        for name, url in db_to_url.items()
    })


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--fin', type=str)
    parser.add_argument('--fout', type=str)
    args = parser.parse_args()

    values = dotenv_values(args.fin)
    values['ROCKET_DATABASES'] = db_string(
        db_to_url={'postgres_database': values['DATABASE_URL']}
    )
    with open(args.fout, 'w') as f:
        f.write('\n'.join(f'{key}={value}' for key, value in values.items()))
