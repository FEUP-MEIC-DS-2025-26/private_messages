import { TextField } from '@mui/material';

export default function PeerSearch({ filter }) {
  return (
      <TextField
        fullWidth
        autoComplete="off"
        name="sender_search"
        size="small"
        placeholder="Sata Andagi"
        onInput={e => {filter(e.target.value);}}
      />
  );
}
