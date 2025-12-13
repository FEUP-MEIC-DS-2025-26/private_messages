import { Box, Typography } from '@mui/material';

export default function ErrorPage({
  message,
  error,
  redirectURL,
}: {
  message: string;
  error: any;
  redirectURL: string;
}) {
  setTimeout(() => window.location.replace(redirectURL), 5000);

  return (
    <Box sx={{ display: 'flex', alignItems: 'center' }}>
      <Typography variant="h3">{message}</Typography>
      <Typography variant="h4">{error}</Typography>
    </Box>
  );
}
