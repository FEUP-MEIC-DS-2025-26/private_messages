import { ChangeEvent } from 'react';
import { TextField } from '@mui/material';

export default function SearchBar({ filter } : { filter: (text: string) => void }) {
  return (
      <TextField
        fullWidth
        autoComplete="off"
        name="search_bar"
        size="small"
        placeholder="Search"
        variant="outlined"
        onChange={(e: ChangeEvent<HTMLInputElement>) => { filter(e.target.value); }}
      />
  );
}
