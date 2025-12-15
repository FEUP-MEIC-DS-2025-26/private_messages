interface NewConversationProps {
  /** The URL that points to the backend. */
  backendURL: string;
  /** The user's JumpSeller ID. */
  userID: number;
}

const handleSellerConversation = async (backend_url: string, my_userid: number, their_userid: number, product_jumpseller_id: number) => {
    const conv_api = backend_url + "/api/chat"
  
    // Login to the "private_messages" service
    const response = await fetch(`${conv_api}/login?id=${my_userid}`, {
      credentials: 'include',
    });

    if (!response.ok) {
      console.error("Failed to login to private_messages service.");
      return;
    }

    // Start a new conversation / Get ID of existing conversation
    const body_params = new URLSearchParams({
      "their_userid": their_userid.toString(), 
      "product_jumpseller_id": product_jumpseller_id.toString(),
    });

    const conversation_id: number = await fetch(
      `${conv_api}/conversation`,
      {
        method: 'POST',
        body: body_params,
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        credentials: 'include',
      }
    )
      .then((res) => res.json())
      .then((x) => x.id);

    // Redirect to private_messages frontend
    const relative_frontend_url = "/chat/" + conversation_id.toString();
    window.location.replace(relative_frontend_url);
  };

export default function NewConversation({ backendURL, userID }: NewConversationProps) {
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        const data = new FormData(e.currentTarget);
        const productId = Number(data.get('productId'));
        const sellerId = Number(data.get('sellerId'));
        console.log({ productId, sellerId });
        handleSellerConversation(backendURL, userID, sellerId, productId);
      }}
    >
      <div>
        <label htmlFor="productId">Product ID</label>
        <input type="text" id="productId" name="productId" required />
      </div>
      <div>
        <label htmlFor="sellerId">Seller ID</label>
        <input type="text" id="sellerId" name="sellerId" required />
      </div>
      <button type="submit">Submit</button>
    </form>
  )
}