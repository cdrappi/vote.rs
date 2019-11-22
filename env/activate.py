import argparse
import json
from typing import Dict

from dotenv import dotenv_values


def format_dict(the_dict: Dict[str, str]):
    inner = ', '.join(f'{key}={value}' for key, value in the_dict.items())
    start, end = '{', '}'
    return f'{start}{inner}{end}'


def db_string(dbs: Dict[str, str]) -> str:
    """ """
    return format_dict({
        name: format_dict({'url': url}) 
        for name, url in dbs.items()
    })


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--fin', type=str)
    parser.add_argument('--fout', type=str)
    args = parser.parse_args()

    values = dotenv_values(args.fin)
    values['ROCKET_DATABASES'] = db_string({'postgres': values['DATABASE_URL']})
    with open(args.fout, 'w') as f:
        f.write('\n'.join(f'{key}={value}' for key, value in values.items()))
